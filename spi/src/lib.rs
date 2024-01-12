//! Generic SPI interface for display drivers

#![no_std]

mod asynch;

use byte_slice_cast::*;
use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_hal::{digital::OutputPin, spi::SpiDevice};

type Result = core::result::Result<(), DisplayError>;

pub(crate) const BUFFER_SIZE: usize = 64;

fn send_u8<SPI>(spi: &mut SPI, words: DataFormat<'_>) -> Result
where
    SPI: SpiDevice,
{
    match words {
        DataFormat::U8(slice) => spi.write(slice).map_err(|_| DisplayError::BusWriteError),
        DataFormat::U16(slice) => spi
            .write(slice.as_byte_slice())
            .map_err(|_| DisplayError::BusWriteError),
        DataFormat::U16LE(slice) => {
            for v in slice.as_mut() {
                *v = v.to_le();
            }
            spi.write(slice.as_byte_slice())
                .map_err(|_| DisplayError::BusWriteError)
        }
        DataFormat::U16BE(slice) => {
            for v in slice.as_mut() {
                *v = v.to_be();
            }
            spi.write(slice.as_byte_slice())
                .map_err(|_| DisplayError::BusWriteError)
        }
        DataFormat::U8Iter(iter) => {
            let mut buf = [0; BUFFER_SIZE];
            let mut i = 0;

            for v in iter.into_iter() {
                buf[i] = v;
                i += 1;

                if i == buf.len() {
                    spi.write(&buf).map_err(|_| DisplayError::BusWriteError)?;
                    i = 0;
                }
            }

            if i > 0 {
                spi.write(&buf[..i])
                    .map_err(|_| DisplayError::BusWriteError)?;
            }

            Ok(())
        }
        DataFormat::U16LEIter(iter) => {
            let mut buf = [0; BUFFER_SIZE];
            let mut i = 0;

            for v in iter.map(u16::to_le) {
                buf[i] = v;
                i += 1;

                if i == buf.len() {
                    spi.write(buf.as_byte_slice())
                        .map_err(|_| DisplayError::BusWriteError)?;
                    i = 0;
                }
            }

            if i > 0 {
                spi.write(buf[..i].as_byte_slice())
                    .map_err(|_| DisplayError::BusWriteError)?;
            }

            Ok(())
        }
        DataFormat::U16BEIter(iter) => {
            let mut buf = [0; BUFFER_SIZE];
            let mut i = 0;
            let len = buf.len();

            for v in iter.map(u16::to_be) {
                buf[i] = v;
                i += 1;

                if i == len {
                    spi.write(buf.as_byte_slice())
                        .map_err(|_| DisplayError::BusWriteError)?;
                    i = 0;
                }
            }

            if i > 0 {
                spi.write(buf[..i].as_byte_slice())
                    .map_err(|_| DisplayError::BusWriteError)?;
            }

            Ok(())
        }
        _ => Err(DisplayError::DataFormatNotImplemented),
    }
}

/// SPI display interface.
///
/// This combines the SPI peripheral and a data/command pin
pub struct SPIInterface<SPI, DC> {
    spi: SPI,
    dc: DC,
}

impl<SPI, DC> SPIInterface<SPI, DC> {
    /// Create new SPI interface for communication with a display driver
    pub fn new(spi: SPI, dc: DC) -> Self {
        Self { spi, dc }
    }

    /// Consume the display interface and return
    /// the underlying peripheral driver and GPIO pins used by it
    pub fn release(self) -> (SPI, DC) {
        (self.spi, self.dc)
    }
}

impl<SPI, DC> WriteOnlyDataCommand for SPIInterface<SPI, DC>
where
    SPI: SpiDevice,
    DC: OutputPin,
{
    fn send_commands(&mut self, cmds: DataFormat<'_>) -> Result {
        // 1 = data, 0 = command
        self.dc.set_low().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        send_u8(&mut self.spi, cmds)
    }

    fn send_data(&mut self, buf: DataFormat<'_>) -> Result {
        // 1 = data, 0 = command
        self.dc.set_high().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        send_u8(&mut self.spi, buf)
    }
}
