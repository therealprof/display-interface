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
    /// Invalid data format selected for interface selected
    InvalidFormatError,
    /// Unable to write to bus
    BusWriteError,
    /// Unable to assert or de-assert data/command switching signal
    DCError,
    /// Unable to assert chip select signal
    CSError,
    /// The requested DataFormat is not implemented by this display interface implementation
    DataFormatNotImplemented,
    /// Unable to assert or de-assert reset signal
    RSError,
    /// Attempted to write to a non-existing pixel outside the display's bounds
    OutOfBoundsError,
}

/// DI specific data format wrapper around slices of various widths
/// Display drivers need to implement non-trivial conversions (e.g. with padding)
/// as the hardware requires.
#[non_exhaustive]
pub enum DataFormat<'a> {
    /// Slice of unsigned bytes
    U8(&'a [u8]),
    /// Slice of unsigned 16bit values with the same endianess as the system, not recommended
    U16(&'a [u16]),
    /// Slice of unsigned 16bit values to be sent in big endian byte order
    U16BE(&'a mut [u16]),
    /// Slice of unsigned 16bit values to be sent in little endian byte order
    U16LE(&'a mut [u16]),
    /// Iterator over unsigned bytes
    U8Iter(&'a mut dyn Iterator<Item = u8>),
    /// Iterator over unsigned 16bit values to be sent in big endian byte order
    U16BEIter(&'a mut dyn Iterator<Item = u16>),
    /// Iterator over unsigned 16bit values to be sent in little endian byte order
    U16LEIter(&'a mut dyn Iterator<Item = u16>),
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
