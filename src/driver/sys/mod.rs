#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub(super) use unix::*;

#[cfg(windows)]
#[allow(unsafe_code)]
mod windows;
#[cfg(windows)]
pub(super) use windows::*;
