//! Generic asynchronous SPI interface for display drivers

use embedded_hal::digital::OutputPin;
use embedded_hal_async::spi::SpiBusWrite;

use display_interface::{AsyncWriteOnlyDataCommand, DataFormat, DisplayError};

use crate::{SPIInterface, SPIInterfaceNoCS};

type Result = core::result::Result<(), DisplayError>;

async fn send_u8<SPI>(spi: &mut SPI, words: DataFormat<'_>) -> Result
where
    SPI: SpiBusWrite,
{
    match words {
        DataFormat::U8(slice) => spi
            .write(slice)
            .await
            .map_err(|_| DisplayError::BusWriteError),
        DataFormat::U16(slice) => {
            use byte_slice_cast::*;
            spi.write(slice.as_byte_slice())
                .await
                .map_err(|_| DisplayError::BusWriteError)
        }
        DataFormat::U16LE(slice) => {
            use byte_slice_cast::*;
            for v in slice.as_mut() {
                *v = v.to_le();
            }
            spi.write(slice.as_byte_slice())
                .await
                .map_err(|_| DisplayError::BusWriteError)
        }
        DataFormat::U16BE(slice) => {
            use byte_slice_cast::*;
            for v in slice.as_mut() {
                *v = v.to_be();
            }
            spi.write(slice.as_byte_slice())
                .await
                .map_err(|_| DisplayError::BusWriteError)
        }
        DataFormat::U8Iter(iter) => {
            let mut buf = [0; 32];
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
            use byte_slice_cast::*;
            let mut buf = [0; 32];
            let mut i = 0;

            for v in iter.map(u16::to_le) {
                buf[i] = v;
                i += 1;

                if i == buf.len() {
                    spi.write(&buf.as_byte_slice())
                        .await
                        .map_err(|_| DisplayError::BusWriteError)?;
                    i = 0;
                }
            }

            if i > 0 {
                spi.write(&buf[..i].as_byte_slice())
                    .await
                    .map_err(|_| DisplayError::BusWriteError)?;
            }

            Ok(())
        }
        DataFormat::U16BEIter(iter) => {
            use byte_slice_cast::*;
            let mut buf = [0; 64];
            let mut i = 0;
            let len = buf.len();

            for v in iter.map(u16::to_be) {
                buf[i] = v;
                i += 1;

                if i == len {
                    spi.write(&buf.as_byte_slice())
                        .await
                        .map_err(|_| DisplayError::BusWriteError)?;
                    i = 0;
                }
            }

            if i > 0 {
                spi.write(&buf[..i].as_byte_slice())
                    .await
                    .map_err(|_| DisplayError::BusWriteError)?;
            }

            Ok(())
        }
        _ => Err(DisplayError::DataFormatNotImplemented),
    }
}

impl<SPI, DC, CS> AsyncWriteOnlyDataCommand for SPIInterface<SPI, DC, CS>
where
    SPI: SpiBusWrite,
    DC: OutputPin,
    CS: OutputPin,
{
    async fn send_commands(&mut self, cmds: DataFormat<'_>) -> Result {
        self.cs_low()?;
        let result = self.spi_no_cs.send_commands(cmds).await;
        self.cs_high().ok();
        result
    }

    async fn send_data(&mut self, buf: DataFormat<'_>) -> Result {
        self.cs_low()?;
        let result = self.spi_no_cs.send_data(buf).await;
        self.cs_high().ok();
        result
    }
}

impl<SPI, DC> AsyncWriteOnlyDataCommand for SPIInterfaceNoCS<SPI, DC>
where
    SPI: SpiBusWrite,
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
