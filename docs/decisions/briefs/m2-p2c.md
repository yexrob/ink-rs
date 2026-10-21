# ink-rs M2 phase 2c: the sync driver

You are working in `<repo>`. Read `CLAUDE.md`, then all of `docs/design.md` (in particular 跨层约定, 编程模型, 事件循环, 终端输出 非 TTY / CI 和 恢复终端), then `docs/ink-tests.md`.

Committed so far:
- `src/runtime/`: components, hooks, effects, Static across frames, `use_cursor`, `use_stdout`, `use_stderr`, `use_window_size`.
- `src/app.rs`, `src/terminal.rs`, `src/output.rs`: the stepping core (`App::render_if_due(now) -> Option<Instant>`), the throttle, interactive and non-interactive output, the alternate screen, `clear`, unmount, external writes, the resize step (M2 phases 2b, 3a, 3b).
- `src/testing.rs`: `testing::render`, `TestTerminal`, `mount`, `TestApp`.

Nothing runs an app on a real terminal yet. This phase adds the sync driver: the public entry point, the event loop, the production `Terminal`, environment detection, cross-thread updates, signals and the panic hook. Read the code above before you change it.

References, all read only:
- The Ink checkout at `<scratchpad>/ink-src/ink`. Read it; never run anything from it. The relevant sources:
  - `src/render.ts` (the `Instance` API), `src/ink.tsx`: the constructor (`signalExit(this.unmount)`, `isInCi`, `resized`), `waitUntilExit` (the `beforeExit` handler: Node fires it when its event loop is empty), `unmount`;
  - `node_modules/is-in-ci/index.js`;
  - `node_modules/chalk/source/vendor/supports-color/index.js` (chalk detects the level of fd 1 with it);
  - `node_modules/signal-exit/index.js` (v3);
  - `test/fixtures/ci.tsx` and the two `components.tsx` tests that run it.
- `<scratchpad>/m2-research/tests-render.md`: the rows of `render only last frame when run in CI` and `render all frames if CI environment variable equals false`.
- `<scratchpad>/expectations/components.tsx.md`: values recorded from a run of Ink's tests.

## 1. What to build

The user decided the four questions below. Do not reopen them; report a problem you find with one.

### Entry point

```rust
let app = ink_rs::render(component(counter, ()), Options::default());
app.wait_until_exit()?;
```

- `pub fn render(element: impl Into<Element>, options: Options) -> Instance` mounts on the process's stdout and runs the first step, so the first frame is written before it returns (Ink `render`).
- `Instance` (Ink's `Instance`):
  - `rerender(&mut self, element)` and `clear(&mut self)`, each followed by a step at the current time;
  - `handle(&self) -> AppHandle` (below);
  - `unmount(self)`; dropping an `Instance` unmounts it as well;
  - `wait_until_exit(self) -> std::io::Result<()>` runs the event loop on the calling thread until the app ends, then unmounts. The result carries the first terminal write error (below). M3 adds exit values; leave room for them, do not add them.
- `Instance` stays `!Send`: the runtime is single-threaded.

### The event loop

- One loop per `wait_until_exit`, on the UI thread. It runs the due steps with the real clock and sleeps until the time `render_if_due` returns, waking early on a message: an `AppHandle` update, a signal, a resize, a handle dropped.
- **Natural end** (Ink: Node fires `beforeExit` when nothing keeps its event loop alive, and `waitUntilExit` unmounts then). The loop returns when all of these hold: the step returned `None` (no work, no pending frame), and no `AppHandle` exists outside the app. M3 adds input subscriptions and M4 timers to this list; keep the condition in one place. An app with no handles ends after its first frame; this matches Ink, where `render(<Text>Hello</Text>)` prints and exits.
- Dropping the last outside `AppHandle` must wake the loop, so it can end. Say in the report how you avoid a lost wakeup between the check and the sleep.
- A terminal write error ends the loop at the next step: unmount (its writes are dropped), and `wait_until_exit` returns the error. Rust ignores SIGPIPE, so a closed pipe shows up as `EPIPE` here.

### `AppHandle` and cross-thread updates

```rust
let handle = cx.use_app();
std::thread::spawn(move || {
    let n = fetch();
    handle.update(move || count.set(n));
});
```

- `AppHandle` is `Clone + Send + Sync`. `update(f)` takes `f: FnOnce() + Send + 'static`, queues it, and wakes the driver. The loop runs queued closures on the UI thread before the step, in the order they were queued, where state writes behave as in an event callback.
- `Scope::use_app()` returns a handle for the component's app; `Instance::handle()` returns one for code outside the tree.
- Holding a handle keeps the loop alive, as an active timer keeps Node's loop alive. A handle the component drops at the end of render does not.
- After the app unmounts, `update` drops the closure.
- `State<T>` becomes `Send + Sync` for every `T`. It holds only ids; reading or writing it off the UI thread panics, as design.md 编程模型 already says. The `use_stdout`/`use_stderr` handles follow the same rule. Check that no other access path makes this unsound, and say so in the report.
- Define `AppHandle` and its queue in the core, with the driver supplying the wakeup (design.md 事件循环: "核心定义唤醒接口"). The tokio driver of M4 must reuse it. The test drivers run queued closures at their next step: `TestApp` at `rerender`/`advance`, `testing::render` at `settle`. `render_to_string` drops them.
- `use_app` gains `exit` in M3. Do not add it.

### The production terminal

- A `Terminal` implementation for the process's stdout and stderr:
  - TTY detection with `std::io::IsTerminal`;
  - size from `crossterm::terminal::size()`, `None` on error (the core falls back to 80×24);
  - each operation the app hands it reaches the fd as one `write_all` that Rust's line buffering does not split at newlines, followed by a flush.
- The first write error is kept; later writes are dropped (the loop then ends, above).

### Environment detection

The core receives resolved values (design.md 非 TTY / CI). The driver resolves them once, in `render`, with functions that take an env lookup so they can be tested without `std::env::set_var` (unsafe in edition 2024).

- **CI** (is-in-ci): `CI` or `CONTINUOUS_INTEGRATION` is set and its value is neither `0` nor `false`. With the TTY check, it sets the default of `Options::interactive`, as `testing::mount` already does with a fixed `false`.
- **Color level**, when `Options::color_level` is `None`: chalk's vendored supports-color for fd 1, mapped 0 → `None`, 1 → `Basic`, 2 → `Ansi256`, 3 → `TrueColor`. Follow its branches in order: `FORCE_COLOR`, Azure DevOps, non-TTY, `TERM=dumb`, CI vendors, TeamCity, `COLORTERM`, the named `TERM` values, `TERM_PROGRAM`, the `TERM` patterns. Leave out the Windows branch (design.md 平台) and the argv flags (`--color`, `--no-color`): a library must not read its host program's arguments. Record both omissions in design.md.

### Signals

Ink unmounts on SIGINT, SIGTERM and SIGHUP through signal-exit, then re-raises the signal. design.md 恢复终端 keeps these three.

- A signal thread (signal-hook's iterator API, no `unsafe`) forwards SIGINT, SIGTERM, SIGHUP and SIGWINCH to every live app's loop.
- **Termination signals during `wait_until_exit`:** the loop unmounts (cleanups run, the terminal is restored), then terminates the process with the original signal (`signal_hook::low_level::emulate_default_handler`).
- **Termination signals outside `wait_until_exit`** (the UI thread is running the program's own code between `render` and `wait_until_exit`, or after it): the loop cannot run, so the signal thread restores the terminal from the shared restore state (below) and terminates with the signal. Cleanups do not run, as when the process is killed.
- **After the last app unmounts**, the three signals regain their default effect, as signal-exit unsubscribes. signal-hook keeps its handler installed after unregistering, so find a safe way (for example `signal_hook::flag::register_conditional_default`) and say in the report how it works.
- **SIGWINCH** runs the resize step of phase 3b with the size the terminal reports. M2 does not read stdin: crossterm's event reader would take input the program may want. M3 moves resize to crossterm's events with input; note this in design.md.

### The panic hook

design.md 跨层约定: drop, the panic hook and termination signals share one restore logic.

- `render` installs a process-wide hook once. It chains the hook that was installed before it.
- **Shared restore state.** After each operation it writes, the app gives the terminal the bytes that would restore it from the current screen state: on the primary screen what `Output::done` would write, without changing `Output`; on the alternate screen `CSI ?1049l CSI ?25h`. The production terminal keeps them where the hook and the signal thread can reach them. Whoever writes them first marks the terminal restored, and the terminal drops every later write, so the restore of the unwinding `Drop` writes nothing. The test terminal ignores them.
- **On a panic**, the hook writes the restore bytes of the apps owned by the panicking thread, then calls the previous hook, so the message lands on the restored primary screen instead of the alternate screen that is about to be left. A panic on another thread leaves the UI running and the terminal alone, unless the crate is built with `panic = "abort"` (`cfg(panic = "abort")`), where every live app is restored first.
- design.md M4 adds the error screen; not in this phase.

### Not in this phase

Input, raw mode, `exit`, Ctrl+C as input, timers and `use_animation`, the tokio driver, SIGTSTP/SIGCONT.

## 2. Dependencies

Add `crossterm = "0.29"` and `signal-hook = "0.3"` (`0.3.18` is in the offline cache; crossterm 0.29 uses the same major). Enable only the features this phase needs. With `CARGO_NET_OFFLINE=true`, if a crate or feature is missing from the cache, stop and report.

## 3. Tests

Global rule for this project: add a test only when you can say why it is needed.

- **Port** the two CI tests of `components.tsx`: `render only last frame when run in CI` and `render all frames if CI environment variable equals false`. They need the loop with an in-memory terminal and a given CI value, which no public API offers; put them next to the loop as unit tests, as the log-update ports sit in `src/output/tests`. The fixture's timer becomes a thread that holds an `AppHandle`, updates every 20 ms through `update`, and drops the handle after the fifth tick; the loop then ends by itself, as Ink's process does. Ink's first test runs with CI set and stdout not a TTY, the second in a PTY: use a non-TTY and a TTY `TestTerminal`. Ink's expectations hold unchanged: CI writes the Static items and only `Counter: 5`; the interactive run writes every counter value.
- **One table test** each for the CI rule and the color rules. Nothing else covers these functions, and a wrong level fails silently. One row per branch, no more.
- No tests for signals, the panic hook or SIGWINCH: they act on the test process itself. List them as manual checks in the report, with the exact steps.
- **`examples/counter.rs`**: Ink's `examples/counter` (a green `{n} tests passed`, incremented every 100 ms; the interval becomes a thread with an `AppHandle`). It serves the manual checks (resize, Ctrl+C, a piped stdout) and is the first example M4 ports.

### `docs/ink-tests.md`

- Update the row of `components.tsx` and its list ("后续 M2" names the two CI tests; they are now ported). Leave the overview counts alone.
- Add the manual checks under the right heading if the document has one; otherwise say so in the report.

### `docs/design.md`

Update where the build decides what design.md leaves open or differs from it: the entry point and natural end (事件循环), `AppHandle` and `State: Send` (编程模型), detection and its omissions (非 TTY / CI), signals, the fallback outside the loop and the panic hook (恢复终端), resize through SIGWINCH until M3. Keep the existing text where it still holds. List each change in the report.

## Rules

- No `unsafe`. Only the two dependencies above.
- No code that exists only to satisfy a test.
- Match the surrounding code:
  - `pub(crate)` for internal items, imports at the top.
  - Comments explain why and cite the Ink source a rule comes from. Do not cite research IDs.
  - Rustdoc on public items; export the new public types and `render` at the crate root.
  - Run `rustfmt --edition 2024` on every file you touch.
- Run only the following. Every test in these targets must pass, apart from `#[ignore]`d ones.
  - `cargo test --lib`
  - `cargo test --test components --test runtime --test render --test cursor --test terminal_resize --test hooks`
  - `cargo build --examples`
  - `cargo clippy --all-targets -- -D warnings`
- Never run `cargo test` without `--lib` or `--test`. The full suite runs at integration.
- Do not run the example on a terminal; the manual checks run at integration.
- If cargo cannot lock the shared target directory, use `CARGO_TARGET_DIR=/tmp/ink-rs-codex-target`, and delete that directory when you are done.
- Your sandbox cannot write `.git`, so leave the work uncommitted.

## Report

1. Proposed commits: message and files. Each commit holds one logical change and builds and passes its tests on its own.
2. The loop: its states, the natural-end condition, how wakeups work and why none is lost, write errors.
3. `AppHandle`: where the queue lives, how handles are counted, how each driver drains it; the `Send` change and why it is sound.
4. Signals and the panic hook: the shared restore state, each path (in the loop, outside it, after unmount, panic on the UI thread, on another thread, `panic = "abort"`), and how the default signal effect returns.
5. Detection: the rules as built, and the omissions.
6. Results of the commands above. Name any failing test, with your diagnosis.
7. The manual checks, step by step; every assertion adapted; where you departed from this brief or from design.md; open questions.
