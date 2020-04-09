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
}

impl<I2C> WriteOnlyDataCommand<u8> for I2CInterface<I2C>
where
    I2C: hal::blocking::i2c::Write,
{
    fn send_commands(&mut self, cmds: &[u8]) -> Result<(), DisplayError> {
        // Copy over given commands to new aray to prefix with command identifier
        let mut writebuf: [u8; 8] = [0; 8];
        writebuf[1..=cmds.len()].copy_from_slice(&cmds[0..cmds.len()]);

        self.i2c
            .write(self.addr, &writebuf[..=cmds.len()])
            .map_err(|_| DisplayError::BusWriteError)
    }

    fn send_data(&mut self, buf: &[u8]) -> Result<(), DisplayError> {
        // No-op if the data buffer is empty
        if buf.is_empty() {
            return Ok(());
        }

        let mut writebuf: [u8; 17] = [0; 17];

        // Data mode
        writebuf[0] = self.data_byte;

        buf.chunks(16)
            .try_for_each(|c| {
                let chunk_len = c.len();

                // Copy over all data from buffer, leaving the data command byte intact
                writebuf[1..=chunk_len].copy_from_slice(c);

                self.i2c.write(self.addr, &writebuf[0..=chunk_len])
            })
            .map_err(|_| DisplayError::BusWriteError)
    }
}
