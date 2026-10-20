/// The driver owns error policy and input; the core writes completed buffers.
pub(crate) trait Terminal {
    // Publish the post-write restore snapshot under the same lock as the write.
    fn write(&mut self, bytes: &[u8], restore: &[u8]);
    fn write_stderr(&mut self, bytes: &[u8], restore: &[u8]);
    fn restore(&mut self, bytes: &[u8]) {
        self.write(bytes, &[]);
    }
    fn take_error(&mut self) -> Option<std::io::Error> {
        None
    }
    fn size(&self) -> Option<(u16, u16)>;
    fn is_tty(&self) -> bool;
    /// Whether stdin supports raw mode.
    fn stdin_is_tty(&self) -> bool;
    /// Switches raw mode; failures are retained for take_error, like write.
    fn set_raw_mode(&mut self, enabled: bool);
}
