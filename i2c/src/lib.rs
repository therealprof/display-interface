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
    I2C: hal::blocking::i2c::Write,
{
    /// Create new I2C interface for communication with a display driver
    pub fn new(i2c: I2C, addr: u8, data_byte: u8) -> Self {
        Self {
            i2c,
            addr,
            data_byte,
        }
    }

    /// Consume the display interface and return
    /// the underlying peripherial driver
    pub fn release(self) -> I2C {
        self.i2c
    }

    fn send_iter<I>(&mut self, first_byte: u8, data: I) -> Result<(), DisplayError>
    where
        I: Iterator<Item = u8>,
    {
        let mut writebuf = [0; 17];
        let mut i = 1;
        let len = writebuf.len();

        // Data/command mode
        writebuf[0] = first_byte;

        for byte in data {
            writebuf[i] = byte;
            i += 1;

            if i == len {
                self.i2c
                    .write(self.addr, &writebuf[0..=len])
                    .map_err(|_| DisplayError::BusWriteError)?;
                i = 1;
            }
        }

        if i > 1 {
            self.i2c
                .write(self.addr, &writebuf[0..=i])
                .map_err(|_| DisplayError::BusWriteError)?;
        }

        Ok(())
    }
}

impl<I2C> WriteOnlyDataCommand for I2CInterface<I2C>
where
    I2C: hal::blocking::i2c::Write,
{
    type Word = u8;

    #[inline]
    fn send_command_iter<I>(&mut self, iter: I) -> Result<(), DisplayError>
    where
        I: Iterator<Item = Self::Word>,
    {
        self.send_iter(0, iter)
    }

    #[inline]
    fn send_data_iter<I>(&mut self, iter: I) -> Result<(), DisplayError>
    where
        I: Iterator<Item = Self::Word>,
    {
        self.send_iter(self.data_byte, iter)
    }
}
