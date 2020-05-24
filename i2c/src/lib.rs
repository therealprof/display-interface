#![no_std]

//! Generic I2C interface for display drivers
use embedded_hal as hal;

use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};

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
}

impl<I2C> WriteOnlyDataCommand for I2CInterface<I2C>
where
    I2C: hal::blocking::i2c::Write,
{
    fn send_commands(&mut self, cmds: DataFormat<'_>) -> Result<(), DisplayError> {
        // Copy over given commands to new aray to prefix with command identifier
        match cmds {
            DataFormat::U8(slice) => {
                let mut writebuf: [u8; 8] = [0; 8];
                writebuf[1..=slice.len()].copy_from_slice(&slice[0..slice.len()]);

                self.i2c
                    .write(self.addr, &writebuf[..=slice.len()])
                    .map_err(|_| DisplayError::BusWriteError)
            }
            _ => Err(DisplayError::DataFormatNotImplemented),
        }
    }

    fn send_data(&mut self, buf: DataFormat<'_>) -> Result<(), DisplayError> {
        match buf {
            DataFormat::U8(slice) => {
                // No-op if the data buffer is empty
                if slice.is_empty() {
                    return Ok(());
                }

                let mut writebuf = [0; 17];

                // Data mode
                writebuf[0] = self.data_byte;

                slice
                    .chunks(16)
                    .try_for_each(|c| {
                        let chunk_len = c.len();

                        // Copy over all data from buffer, leaving the data command byte intact
                        writebuf[1..=chunk_len].copy_from_slice(c);

                        self.i2c.write(self.addr, &writebuf[0..=chunk_len])
                    })
                    .map_err(|_| DisplayError::BusWriteError)
            }
            DataFormat::U8Iter(iter) => {
                let mut writebuf = [0; 17];
                let mut i = 1;
                let len = writebuf.len();

                // Data mode
                writebuf[0] = self.data_byte;

                for byte in iter.into_iter() {
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
            _ => Err(DisplayError::DataFormatNotImplemented),
        }
    }
}
