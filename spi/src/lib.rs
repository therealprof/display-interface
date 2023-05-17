#![no_std]
#![cfg_attr(
    all(feature = "async", feature = "nightly"),
    allow(incomplete_features),
    feature(async_fn_in_trait, impl_trait_projections)
)]

//! Generic SPI interface for display drivers

#[cfg(all(feature = "async", not(feature = "nightly")))]
extern crate alloc;

#[cfg(feature = "async")]
pub mod asynch;

use embedded_hal::{digital::OutputPin, spi::SpiBusWrite};

use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};

type Result = core::result::Result<(), DisplayError>;

fn send_u8<SPI>(spi: &mut SPI, words: DataFormat<'_>) -> Result
where
    SPI: SpiBusWrite,
{
    match words {
        DataFormat::U8(slice) => spi.write(slice).map_err(|_| DisplayError::BusWriteError),
        DataFormat::U16(slice) => {
            use byte_slice_cast::*;
            spi.write(slice.as_byte_slice())
                .map_err(|_| DisplayError::BusWriteError)
        }
        DataFormat::U16LE(slice) => {
            use byte_slice_cast::*;
            for v in slice.as_mut() {
                *v = v.to_le();
            }
            spi.write(slice.as_byte_slice())
                .map_err(|_| DisplayError::BusWriteError)
        }
        DataFormat::U16BE(slice) => {
            use byte_slice_cast::*;
            for v in slice.as_mut() {
                *v = v.to_be();
            }
            spi.write(slice.as_byte_slice())
                .map_err(|_| DisplayError::BusWriteError)
        }
        DataFormat::U8Iter(iter) => {
            let mut buf = [0; 32];
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
            use byte_slice_cast::*;
            let mut buf = [0; 32];
            let mut i = 0;

            for v in iter.map(u16::to_le) {
                buf[i] = v;
                i += 1;

                if i == buf.len() {
                    spi.write(&buf.as_byte_slice())
                        .map_err(|_| DisplayError::BusWriteError)?;
                    i = 0;
                }
            }

            if i > 0 {
                spi.write(&buf[..i].as_byte_slice())
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
                        .map_err(|_| DisplayError::BusWriteError)?;
                    i = 0;
                }
            }

            if i > 0 {
                spi.write(&buf[..i].as_byte_slice())
                    .map_err(|_| DisplayError::BusWriteError)?;
            }

            Ok(())
        }
        _ => Err(DisplayError::DataFormatNotImplemented),
    }
}

/// SPI display interface.
///
/// This combines the SPI peripheral and a data/command as well as a chip-select pin
pub struct SPIInterface<SPI, DC, CS> {
    spi_no_cs: SPIInterfaceNoCS<SPI, DC>,
    cs: CS,
}

impl<SPI, DC, CS> SPIInterface<SPI, DC, CS>
where
    CS: OutputPin,
{
    pub(crate) fn cs_low(&mut self) -> Result {
        self.cs.set_low().map_err(|_| DisplayError::CSError)?;
        Ok(())
    }

    pub(crate) fn cs_high(&mut self) -> Result {
        self.cs.set_high().map_err(|_| DisplayError::CSError)?;
        Ok(())
    }
}

impl<SPI, DC, CS> SPIInterface<SPI, DC, CS>
where
    SPI: SpiBusWrite,
    DC: OutputPin,
    CS: OutputPin,
{
    /// Create new SPI interface for communication with a display driver
    pub fn new(spi: SPI, dc: DC, cs: CS) -> Self {
        Self {
            spi_no_cs: SPIInterfaceNoCS::new(spi, dc),
            cs,
        }
    }

    /// Consume the display interface and return
    /// the underlying peripherial driver and GPIO pins used by it
    pub fn release(self) -> (SPI, DC, CS) {
        let (spi, dc) = self.spi_no_cs.release();
        (spi, dc, self.cs)
    }

    fn with_cs(&mut self, f: impl FnOnce(&mut SPIInterfaceNoCS<SPI, DC>) -> Result) -> Result {
        // Assert chip select pin
        self.cs_low().map_err(|_| DisplayError::CSError)?;

        let result = f(&mut self.spi_no_cs);

        // Deassert chip select pin
        self.cs_high().ok();

        result
    }
}

impl<SPI, DC, CS> WriteOnlyDataCommand for SPIInterface<SPI, DC, CS>
where
    SPI: SpiBusWrite,
    DC: OutputPin,
    CS: OutputPin,
{
    fn send_commands(&mut self, cmds: DataFormat<'_>) -> Result {
        self.with_cs(|spi_no_cs| spi_no_cs.send_commands(cmds))
    }

    fn send_data(&mut self, buf: DataFormat<'_>) -> Result {
        self.with_cs(|spi_no_cs| spi_no_cs.send_data(buf))
    }
}

/// SPI display interface.
///
/// This combines the SPI peripheral and a data/command pin
pub struct SPIInterfaceNoCS<SPI, DC> {
    spi: SPI,
    dc: DC,
}

impl<SPI, DC> SPIInterfaceNoCS<SPI, DC>
where
    SPI: SpiBusWrite,
    DC: OutputPin,
{
    /// Create new SPI interface for communication with a display driver
    pub fn new(spi: SPI, dc: DC) -> Self {
        Self { spi, dc }
    }

    /// Consume the display interface and return
    /// the underlying peripherial driver and GPIO pins used by it
    pub fn release(self) -> (SPI, DC) {
        (self.spi, self.dc)
    }
}

impl<SPI, DC> WriteOnlyDataCommand for SPIInterfaceNoCS<SPI, DC>
where
    SPI: SpiBusWrite,
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
