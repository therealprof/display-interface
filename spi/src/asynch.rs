//! Generic asynchronous SPI interface for display drivers

use byte_slice_cast::*;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::spi::SpiDevice;

use display_interface::{AsyncWriteOnlyDataCommand, DataFormat, DisplayError};

use crate::{SpiInterface, BUFFER_SIZE};

type Result = core::result::Result<(), DisplayError>;

async fn send_u8<SPI>(spi: &mut SPI, words: DataFormat<'_>) -> Result
where
    SPI: SpiDevice,
{
    match words {
        DataFormat::U8(slice) => spi
            .write(slice)
            .await
            .map_err(|_| DisplayError::BusWriteError),
        DataFormat::U16(slice) => spi
            .write(slice.as_byte_slice())
            .await
            .map_err(|_| DisplayError::BusWriteError),
        DataFormat::U16LE(slice) => {
            for v in slice.as_mut() {
                *v = v.to_le();
            }
            spi.write(slice.as_byte_slice())
                .await
                .map_err(|_| DisplayError::BusWriteError)
        }
        DataFormat::U16BE(slice) => {
            for v in slice.as_mut() {
                *v = v.to_be();
            }
            spi.write(slice.as_byte_slice())
                .await
                .map_err(|_| DisplayError::BusWriteError)
        }
        DataFormat::U8Iter(iter) => {
            let mut buf = [0; BUFFER_SIZE];
            let mut i = 0;

            for v in iter.into_iter() {
                buf[i] = v;
                i += 1;

                if i == buf.len() {
                    spi.write(&buf)
                        .await
                        .map_err(|_| DisplayError::BusWriteError)?;
                    i = 0;
                }
            }

            if i > 0 {
                spi.write(&buf[..i])
                    .await
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
                        .await
                        .map_err(|_| DisplayError::BusWriteError)?;
                    i = 0;
                }
            }

            if i > 0 {
                spi.write(buf[..i].as_byte_slice())
                    .await
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
                        .await
                        .map_err(|_| DisplayError::BusWriteError)?;
                    i = 0;
                }
            }

            if i > 0 {
                spi.write(buf[..i].as_byte_slice())
                    .await
                    .map_err(|_| DisplayError::BusWriteError)?;
            }

            Ok(())
        }
        _ => Err(DisplayError::DataFormatNotImplemented),
    }
}

impl<SPI, DC> AsyncWriteOnlyDataCommand for SpiInterface<SPI, DC>
where
    SPI: SpiDevice,
    DC: OutputPin,
{
    async fn send_commands(&mut self, cmds: DataFormat<'_>) -> Result {
        // 1 = data, 0 = command
        self.dc.set_low().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        send_u8(&mut self.spi, cmds).await
    }

    async fn send_data(&mut self, buf: DataFormat<'_>) -> Result {
        // 1 = data, 0 = command
        self.dc.set_high().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        send_u8(&mut self.spi, buf).await
    }
}
