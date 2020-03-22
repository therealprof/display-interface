#![no_std]

pub trait WriteOnlyDataCommand {
    /// Interface error type
    type Error;

    /// Send a batch of commands to display
    fn send_commands(&mut self, cmd: &[u8]) -> Result<(), Self::Error>;

    /// Send pixel data to display
    fn send_data(&mut self, buf: &[u8]) -> Result<(), Self::Error>;
}
