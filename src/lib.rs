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
#[non_exhaustive]
pub enum DisplayError {
    /// Unable to write to bus
    BusWriteError,
    /// Unable to assert or de-assert data/command switching signal
    DCError,
    /// Unable to assert chip select signal
    CSError,
    /// Unable to assert or de-assert reset signal
    RSError,
    /// Attempted to write to a non-existing pixel outside the display's bounds
    OutOfBoundsError,
}

/// This trait implements a write-only interface for a display which has separate data and command
/// modes. It is the responsibility of implementations to activate the correct mode in their
/// implementation when corresponding method is called.
pub trait WriteOnlyDataCommand {
    type Word: Copy;

    fn send_command_iter(
        &mut self,
        iter: impl Iterator<Item = Self::Word>,
    ) -> Result<(), DisplayError>;

    fn send_data_iter(
        &mut self,
        iter: impl Iterator<Item = Self::Word>,
    ) -> Result<(), DisplayError>;

    #[inline]
    fn send_command_slice(&mut self, slice: &[Self::Word]) -> Result<(), DisplayError> {
        self.send_command_iter(slice.iter().copied())
    }

    #[inline]
    fn send_data_slice(&mut self, slice: &[Self::Word]) -> Result<(), DisplayError> {
        self.send_data_iter(slice.iter().copied())
    }
}
