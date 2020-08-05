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
    // Unable to read from bus
    BusReadError,
    /// Unable to assert or de-assert data/command switching signal
    DCError,
    /// Unable to assert chip select signal
    CSError,
    /// The requested DataFormat is not implemented by this display interface implementation
    DataFormatNotImplemented,
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

#[derive(Clone, Debug)]
pub enum WriteMode {
    Data,
    Command,
}

pub trait ReadWriteInterface<DataFormat> {
    fn write(&mut self, mode: WriteMode, buf: &[DataFormat]) -> Result<(), DisplayError> {
        self.write_iter(mode, &mut buf.into_iter())
    }

    fn read(&mut self, buf: &mut [DataFormat]) -> Result<(), DisplayError> {
        let mut n = 0;
        self.read_stream(&mut |b| {
            if n == buf.len() {
                return false;
            }
            buf[n] = b;
            n += 1;
            true
        })
    }

    fn read_stream(&mut self, f: &mut dyn FnMut(DataFormat) -> bool) -> Result<(), DisplayError>;

    fn write_iter(
        &mut self,
        mode: WriteMode,
        iter: &mut dyn Iterator<Item = &DataFormat>,
    ) -> Result<(), DisplayError>;
}

pub struct ReadIterator<'a, DataFormat> {
    rw: &'a mut dyn ReadWriteInterface<DataFormat>,
}

impl<'a, DataFormat> ReadIterator<'a, DataFormat> {
    fn new(rw: &'a mut dyn ReadWriteInterface<DataFormat>) -> ReadIterator<'a, DataFormat> {
        ReadIterator { rw: rw }
    }
}

impl<'a, DataFormat> Iterator for ReadIterator<'a, DataFormat>
where
    DataFormat: Default,
{
    type Item = Result<DataFormat, DisplayError>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut next: DataFormat = Default::default();
        match self.rw.read_stream(&mut |b| {
            next = b;
            false
        }) {
            Ok(_) => Some(Ok(next)),
            Err(e) => Some(Err(e)),
        }
    }
}

impl<'a, DataFormat> IntoIterator for &'a mut dyn ReadWriteInterface<DataFormat>
where
    DataFormat: Default,
{
    type Item = Result<DataFormat, DisplayError>;
    type IntoIter = ReadIterator<'a, DataFormat>;

    fn into_iter(self) -> Self::IntoIter {
        ReadIterator::new(self)
    }
}
