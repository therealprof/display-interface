#![no_std]

//! A generic display interface
//!
//! This crate contains an error type and traits to implement for bus interface drivers drivers to
//! be consumed by display drivers. It abstracts over the different communication methods available
//! to drive a display and allows a driver writer to focus on driving the display itself and only
//! have to implement a single interface.

pub mod prelude;

/// A ubiquitous error type for all kinds of problems which could happen when communicating with a
/// display
#[derive(Clone, Debug)]
pub enum DisplayError {
    /// Unable to write to bus
    BusWriteError,
    /// Unable to assert or de-assert data/command switching signal
    DCError,
    /// Unable to assert chip select signal
    CSError,
}

pub enum DataFormat<'a> {
    U8(&'a [u8]),
    U16(&'a [u16]),
}

impl<'a> From<&'a [u8]> for DataFormat<'a> {
    fn from(arr_ref: &'a [u8]) -> Self {
        Self::U8(arr_ref)
    }
}
impl<'a> From<&'a [u16]> for DataFormat<'a> {
    fn from(arr_ref: &'a [u16]) -> Self {
        Self::U16(arr_ref)
    }
}

/// This trait implements a write-only interface for a display which has separate data and command
/// modes. It is the responsibility of implementations to activate the correct mode in their
/// implementation when corresponding method is called.
pub trait WriteOnlyDataCommand {
    /// Send a batch of commands to display
    fn send_commands(&mut self, cmd: DataFormat<'_>) -> Result<(), DisplayError>;

    /// Send pixel data to display
    fn send_data(&mut self, buf: DataFormat<'_>) -> Result<(), DisplayError>;
}
