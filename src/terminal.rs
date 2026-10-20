/// The driver owns error policy and input; the core writes completed buffers.
pub(crate) trait Terminal {
    fn write(&mut self, bytes: &[u8]);
    fn write_stderr(&mut self, bytes: &[u8]);
    fn size(&self) -> Option<(u16, u16)>;
    fn is_tty(&self) -> bool;
}
