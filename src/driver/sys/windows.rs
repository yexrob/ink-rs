use std::fs::File;
use std::io::{self, Write};
use std::os::windows::io::{AsHandle, AsRawHandle, BorrowedHandle, FromRawHandle, OwnedHandle};
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Once, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use windows_sys::Win32::Foundation::{WAIT_FAILED, WAIT_OBJECT_0};
use windows_sys::Win32::System::Console::{
    CONSOLE_SCREEN_BUFFER_INFO, ENABLE_PROCESSED_OUTPUT, ENABLE_VIRTUAL_TERMINAL_INPUT,
    ENABLE_VIRTUAL_TERMINAL_PROCESSING, GetConsoleMode, GetConsoleScreenBufferInfo, INPUT_RECORD,
    KEY_EVENT, ReadConsoleInputW, SetConsoleCtrlHandler, SetConsoleMode, WriteConsoleW,
};
use windows_sys::Win32::System::Threading::{
    CreateEventW, INFINITE, SetEvent, WaitForMultipleObjects,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_MENU;

use super::super::stdin::Route;
use super::super::{process, stdin};

pub(in crate::driver) type SavedMode = u32;
pub(in crate::driver) type Termination = u32;
pub(in crate::driver) const JOB_CONTROL: bool = false;

pub(in crate::driver) struct Output {
    file: File,
    console: bool,
}

impl Output {
    fn new(handle: BorrowedHandle<'_>) -> io::Result<Self> {
        enable_vt_output();
        let file = File::from(handle.try_clone_to_owned()?);
        let console = mode(file.as_handle()).is_ok();
        Ok(Self { file, console })
    }
}

impl Write for Output {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if !self.console {
            return self.file.write(bytes);
        }
        let units: Vec<u16> = String::from_utf8_lossy(bytes).encode_utf16().collect();
        let mut remaining = units.as_slice();
        while !remaining.is_empty() {
            let len = remaining.len().min(u32::MAX as usize) as u32;
            let mut written = 0;
            // SAFETY: file owns a valid handle, len fits the UTF-16 buffer, and written is initialized.
            let result = unsafe {
                WriteConsoleW(
                    self.file.as_raw_handle(),
                    remaining.as_ptr(),
                    len,
                    &mut written,
                    ptr::null(),
                )
            };
            if result == 0 {
                return Err(io::Error::last_os_error());
            }
            if written == 0 {
                return Err(io::ErrorKind::WriteZero.into());
            }
            remaining = &remaining[written as usize..];
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

// Duplicated handles bypass std::io::Stdout's LineWriter: a synchronized
// operation containing newlines reaches Output::write_all as one buffer.
pub(in crate::driver) fn stdout() -> io::Result<Output> {
    Output::new(io::stdout().as_handle())
}

pub(in crate::driver) fn stderr() -> io::Result<Output> {
    Output::new(io::stderr().as_handle())
}

fn mode(handle: BorrowedHandle<'_>) -> io::Result<u32> {
    let mut mode = 0;
    // SAFETY: handle is borrowed for the call and mode is an initialized out-parameter.
    if unsafe { GetConsoleMode(handle.as_raw_handle(), &mut mode) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(mode)
}

fn set_mode(handle: BorrowedHandle<'_>, mode: u32) -> io::Result<()> {
    // SAFETY: handle is borrowed for the duration of the call.
    if unsafe { SetConsoleMode(handle.as_raw_handle(), mode) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn enable_vt_output() {
    static VT: Once = Once::new();
    VT.call_once(|| {
        let stdout = io::stdout();
        let stderr = io::stderr();
        for handle in [stdout.as_handle(), stderr.as_handle()] {
            if let Ok(mode) = mode(handle) {
                // libuv leaves VT enabled at unmount. Keep LF's return to column 0;
                // do not add DISABLE_NEWLINE_AUTO_RETURN or change the code page.
                let _ = set_mode(
                    handle,
                    mode | ENABLE_PROCESSED_OUTPUT | ENABLE_VIRTUAL_TERMINAL_PROCESSING,
                );
            }
        }
    });
}

pub(in crate::driver) fn stdout_is_tty() -> bool {
    enable_vt_output();
    mode(io::stdout().as_handle()).is_ok_and(|mode| mode & ENABLE_VIRTUAL_TERMINAL_PROCESSING != 0)
}

pub(in crate::driver) fn stdin_is_tty() -> bool {
    mode(io::stdin().as_handle()).is_ok()
}

pub(in crate::driver) fn size() -> Option<(u16, u16)> {
    let stdout = io::stdout();
    let mut info = CONSOLE_SCREEN_BUFFER_INFO::default();
    // SAFETY: stdout supplies the handle and info is an initialized out-parameter.
    if unsafe { GetConsoleScreenBufferInfo(stdout.as_raw_handle(), &mut info) } == 0 {
        return None;
    }
    Some((
        info.dwSize.X as u16,
        (i32::from(info.srWindow.Bottom) - i32::from(info.srWindow.Top) + 1) as u16,
    ))
}

pub(in crate::driver) fn enter_raw_mode() -> io::Result<SavedMode> {
    let stdin = io::stdin();
    let saved = mode(stdin.as_handle())?;
    // libuv UV_TTY_MODE_RAW_VT: no processed input, so Ctrl+C is a byte.
    // libuv adds ENABLE_WINDOW_INPUT for size records; the resize poll replaces them.
    // Omitting ENABLE_EXTENDED_FLAGS leaves the user's quick-edit setting alone.
    set_mode(stdin.as_handle(), ENABLE_VIRTUAL_TERMINAL_INPUT)?;
    Ok(saved)
}

pub(in crate::driver) fn restore_mode(saved: &SavedMode, _emergency: bool) -> io::Result<()> {
    set_mode(io::stdin().as_handle(), *saved)
}

pub(in crate::driver) struct Reader {
    wake: Arc<OwnedHandle>,
    thread: JoinHandle<()>,
}

impl Reader {
    pub(in crate::driver) fn start(route: Arc<Mutex<Route>>) -> io::Result<Self> {
        let input = io::stdin().as_handle().try_clone_to_owned()?;
        // SAFETY: null security attributes and name request an unnamed event with default security.
        let event = unsafe { CreateEventW(ptr::null(), 1, 0, ptr::null()) };
        if event.is_null() {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: CreateEventW returned a valid owned handle; ownership transfers once.
        let wake = Arc::new(unsafe { OwnedHandle::from_raw_handle(event) });
        let wake_read = wake.clone();
        let thread = thread::Builder::new()
            .name("ink-stdin".into())
            .spawn(move || read(input, wake_read, route))?;
        Ok(Self { wake, thread })
    }

    pub(in crate::driver) fn stop(self) -> io::Result<()> {
        // SAFETY: wake owns the event handle until the reader has joined.
        let result = unsafe { SetEvent(self.wake.as_raw_handle()) };
        let error = (result == 0).then(io::Error::last_os_error);
        if self.thread.join().is_err() {
            return Err(io::Error::other("stdin reader panicked"));
        }
        error.map_or(Ok(()), Err)
    }
}

fn read(input: OwnedHandle, wake: Arc<OwnedHandle>, route: Arc<Mutex<Route>>) {
    // Put wake first so stopping wins when both handles are signalled.
    let handles = [wake.as_raw_handle(), input.as_raw_handle()];
    let mut records = [INPUT_RECORD::default(); 128];
    let mut high_surrogate = None;
    loop {
        // SAFETY: both owned handles outlive the wait; the array contains two handles.
        let result = unsafe { WaitForMultipleObjects(2, handles.as_ptr(), 0, INFINITE) };
        if result == WAIT_OBJECT_0 {
            return;
        }
        if result != WAIT_OBJECT_0 + 1 {
            let error = if result == WAIT_FAILED {
                io::Error::last_os_error()
            } else {
                io::Error::other("unexpected console wait result")
            };
            stdin::ended(&route, Some(error));
            return;
        }
        let mut count = 0;
        // SAFETY: input owns a valid handle, records has the stated length, and count is initialized.
        let result = unsafe {
            ReadConsoleInputW(
                input.as_raw_handle(),
                records.as_mut_ptr(),
                records.len() as u32,
                &mut count,
            )
        };
        if result == 0 {
            stdin::ended(&route, Some(io::Error::last_os_error()));
            return;
        }
        let bytes = decode_records(&records[..count as usize], &mut high_surrogate);
        if !bytes.is_empty() {
            stdin::input(&route, &bytes);
        }
    }
}

fn decode_records(records: &[INPUT_RECORD], high_surrogate: &mut Option<u16>) -> Vec<u8> {
    let mut text = String::new();
    for record in records {
        if u32::from(record.EventType) != KEY_EVENT {
            continue;
        }
        // SAFETY: KEY_EVENT selects the initialized KeyEvent member of INPUT_RECORD.
        let key = unsafe { record.Event.KeyEvent };
        // SAFETY: ReadConsoleInputW supplies the UnicodeChar member, an initialized u16.
        let unit = unsafe { key.uChar.UnicodeChar };
        if unit == 0 || (key.bKeyDown == 0 && key.wVirtualKeyCode != VK_MENU) {
            continue;
        }
        if (0xd800..=0xdbff).contains(&unit) {
            if high_surrogate.replace(unit).is_some() {
                text.push(char::REPLACEMENT_CHARACTER);
            }
            continue;
        }
        let high = high_surrogate.take();
        // Repeat the decoded character, not the individual halves of a surrogate pair.
        // VT input supplies its own escape sequences; Alt needs no ESC prefix here.
        for _ in 0..key.wRepeatCount.max(1) {
            text.extend(
                char::decode_utf16(high.into_iter().chain([unit]))
                    .map(|character| character.unwrap_or(char::REPLACEMENT_CHARACTER)),
            );
        }
    }
    text.into_bytes()
}

static DEFAULT_SIGNALS: OnceLock<Arc<AtomicBool>> = OnceLock::new();

extern "system" fn control_handler(control: u32) -> i32 {
    if DEFAULT_SIGNALS
        .get()
        .is_none_or(|default| default.load(Ordering::SeqCst))
    {
        return 0;
    }
    process::signal(control);
    // CLOSE/LOGOFF/SHUTDOWN terminate the process when this handler returns.
    // Let the last UI registration run cleanups and call terminate instead.
    loop {
        thread::park();
    }
}

pub(in crate::driver) fn install_events(
    default_signals: Arc<AtomicBool>,
) -> &'static io::Result<()> {
    static EVENTS: OnceLock<io::Result<()>> = OnceLock::new();
    EVENTS.get_or_init(|| {
        let _ = DEFAULT_SIGNALS.set(default_signals);
        // SAFETY: the handler has the required ABI and remains valid for the process lifetime.
        if unsafe { SetConsoleCtrlHandler(Some(control_handler), 1) } == 0 {
            return Err(io::Error::last_os_error());
        }
        // SetWinEventHook needs user32, a message loop, and undocumented ntdll
        // NtQueryInformationProcess to find conhost. Poll without those dependencies.
        let mut previous = size();
        thread::Builder::new()
            .name("ink-resize".into())
            .spawn(move || {
                loop {
                    thread::sleep(Duration::from_millis(100));
                    let current = size();
                    if current != previous {
                        previous = current;
                        process::resize();
                    }
                }
            })?;
        Ok(())
    })
}

pub(in crate::driver) fn terminate(_control: Termination) -> ! {
    std::process::exit(0xc000013a_u32 as i32); // STATUS_CONTROL_C_EXIT
}

pub(in crate::driver) fn stop_process() {
    // Windows has no job control; mount disables suspend_on_ctrl_z.
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_sys::Win32::System::Console::{
        INPUT_RECORD_0, KEY_EVENT_RECORD, KEY_EVENT_RECORD_0,
    };

    // ConPTY cannot force split reads; the decoder must retain the high surrogate
    // while dropping releases and accepting Alt+numpad composition's key-up.
    #[test]
    fn records_decode_across_reads_and_filter_key_up() {
        let key = |unit, down, code| INPUT_RECORD {
            EventType: KEY_EVENT as u16,
            Event: INPUT_RECORD_0 {
                KeyEvent: KEY_EVENT_RECORD {
                    bKeyDown: down,
                    wRepeatCount: 1,
                    wVirtualKeyCode: code,
                    uChar: KEY_EVENT_RECORD_0 { UnicodeChar: unit },
                    ..Default::default()
                },
            },
        };
        let mut high = None;
        assert!(decode_records(&[key(0xd83d, 1, 0)], &mut high).is_empty());
        let bytes = decode_records(
            &[
                key(u16::from(b'x'), 0, 0),
                key(0xde00, 1, 0),
                key(u16::from(b'y'), 0, 0),
                key(0x00e9, 0, VK_MENU),
            ],
            &mut high,
        );
        assert_eq!(bytes, "\u{1f600}\u{e9}".as_bytes());
        assert_eq!(high, None);
    }
}
