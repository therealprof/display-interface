#![no_std]

//! Generic SPI interface for display drivers

use embedded_hal::{
    digital::blocking::OutputPin,
    spi::blocking::{SpiBusWrite, SpiDevice},
};

use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};

type Result<T = (), E = DisplayError> = core::result::Result<T, E>;

fn send_u8_iter<SPI: SpiBusWrite>(
    mut spi: SPI,
    iter: impl Iterator<Item = u8>,
) -> Result<(), SPI::Error> {
    let mut buf = [0; 32];
    let mut i = 0;

    for v in iter {
        buf[i] = v;
        i += 1;

        if i == buf.len() {
            spi.write(&buf)?;
            i = 0;
        }
    }

    if i > 0 {
        spi.write(&buf[..i])?;
    }

    Ok(())
}

fn send_dataformat<SPI: SpiBusWrite>(mut spi: SPI, data: DataFormat<'_>) -> Result {
    use byte_slice_cast::AsByteSlice;

    match data {
        DataFormat::U8(slice) => spi.write(slice),
        DataFormat::U16(slice) => spi.write(slice.as_byte_slice()),
        DataFormat::U16LE(slice) => {
            for v in slice.iter_mut() {
                *v = v.to_le();
            }
            spi.write(slice.as_byte_slice())
        }
        DataFormat::U16BE(slice) => {
            for v in slice.iter_mut() {
                *v = v.to_be();
            }
            spi.write(slice.as_byte_slice())
        }
        DataFormat::U8Iter(iter) => send_u8_iter(spi, iter),
        DataFormat::U16LEIter(iter) => send_u8_iter(spi, iter.flat_map(u16::to_le_bytes)),
        DataFormat::U16BEIter(iter) => send_u8_iter(spi, iter.flat_map(u16::to_be_bytes)),

        _ => return Err(DisplayError::DataFormatNotImplemented),
    }
    .map_err(|_| DisplayError::BusWriteError)
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
    SPI: SpiDevice,
    SPI::Bus: SpiBusWrite,
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
        self.cs.set_low().map_err(|_| DisplayError::CSError)?;

        let result = f(&mut self.spi_no_cs);

        // Deassert chip select pin
        self.cs.set_high().ok();

        result
    }
}

impl<SPI, DC, CS> WriteOnlyDataCommand for SPIInterface<SPI, DC, CS>
where
    SPI: SpiDevice,
    SPI::Bus: SpiBusWrite,
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
    SPI: SpiDevice,
    SPI::Bus: SpiBusWrite,
    DC: OutputPin,
{
    /// Create new SPI interface for communciation with a display driver
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
    SPI: SpiDevice,
    SPI::Bus: SpiBusWrite,
    DC: OutputPin,
{
    fn send_commands(&mut self, cmds: DataFormat<'_>) -> Result {
        // 1 = data, 0 = command
        self.dc.set_low().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        self.spi
            .transaction(|spi| Ok(send_dataformat(spi, cmds)))
            .map_err(|_| DisplayError::BusWriteError)?
    }

    fn send_data(&mut self, buf: DataFormat<'_>) -> Result {
        // 1 = data, 0 = command
        self.dc.set_high().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        self.spi
            .transaction(|spi| Ok(send_dataformat(spi, buf)))
            .map_err(|_| DisplayError::BusWriteError)?
    }
}
