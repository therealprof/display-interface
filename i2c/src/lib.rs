#![no_std]

//! Generic I2C interface for display drivers
use embedded_hal as hal;

use display_interface::{DisplayError, WriteOnlyDataCommand};

/// I2C communication interface
pub struct I2CInterface<I2C> {
    i2c: I2C,
    addr: u8,
    data_byte: u8,
}

impl<I2C> I2CInterface<I2C>
where
    I2C: hal::blocking::i2c::WriteIter,
{
    /// Create new I2C interface for communication with a display driver
    pub fn new(i2c: I2C, addr: u8, data_byte: u8) -> Self {
        Self {
            i2c,
            addr,
            data_byte,
        }
    }
}

impl<I2C> WriteOnlyDataCommand for I2CInterface<I2C>
where
    I2C: hal::blocking::i2c::WriteIter,
{
    type Width = u8;
    
    fn send_commands<S>(&mut self, cmds: S) -> Result<(), DisplayError>
    where
        S: IntoIterator<Item = Self::Width>,
    {
        // Create prefixed iterator with command prefix as 1st u8 value
        let prefixed = core::iter::once(0u8).chain(cmds);

        self.i2c
            .write(self.addr, prefixed)
            .map_err(|_| DisplayError::BusWriteError)
    }

    fn send_data<S>(&mut self, buf: S) -> Result<(), DisplayError>
    where
        S: IntoIterator<Item = Self::Width>,
    {
        // Data mode
        let prefixed = core::iter::once(self.data_byte).chain(buf);

        self.i2c.write(self.addr, prefixed)
            .map_err(|_| DisplayError::BusWriteError)
    }
}
