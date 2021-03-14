#![no_std]

//! Generic SPI interface for display drivers

use embedded_hal as hal;
use hal::digital::v2::OutputPin;

use display_interface::{DisplayError, WriteOnlyDataCommand};

fn send_iter(
    spi: &mut impl hal::blocking::spi::Write<u8>,
    iter: impl Iterator<Item = u8>,
) -> Result<(), DisplayError> {
    let mut buf = [0; 32];
    let mut i = 0;

    for v in iter {
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

/// SPI display interface.
///
/// This combines the SPI peripheral and a data/command as well as a chip-select pin
pub struct SPIInterface<SPI, DC, CS> {
    spi: SPI,
    dc: DC,
    cs: CS,
}

impl<SPI, DC, CS> SPIInterface<SPI, DC, CS>
where
    SPI: hal::blocking::spi::Write<u8>,
    DC: OutputPin,
    CS: OutputPin,
{
    /// Create new SPI interface for communication with a display driver
    pub fn new(spi: SPI, dc: DC, cs: CS) -> Self {
        Self { spi, dc, cs }
    }

    /// Consume the display interface and return
    /// the underlying peripherial driver and GPIO pins used by it
    pub fn release(self) -> (SPI, DC, CS) {
        (self.spi, self.dc, self.cs)
    }
}

impl<SPI, DC, CS> WriteOnlyDataCommand for SPIInterface<SPI, DC, CS>
where
    SPI: hal::blocking::spi::Write<u8>,
    DC: OutputPin,
    CS: OutputPin,
{
    type Word = u8;

    #[inline]
    fn send_command_iter(
        &mut self,
        iter: impl Iterator<Item = Self::Word>,
    ) -> Result<(), DisplayError> {
        // Assert chip select pin
        self.cs.set_low().map_err(|_| DisplayError::CSError)?;

        // 1 = data, 0 = command
        self.dc.set_low().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        let err = send_iter(&mut self.spi, iter);

        // Deassert chip select pin
        self.cs.set_high().ok();

        err
    }

    #[inline]
    fn send_data_iter(
        &mut self,
        iter: impl Iterator<Item = Self::Word>,
    ) -> Result<(), DisplayError> {
        // Assert chip select pin
        self.cs.set_low().map_err(|_| DisplayError::CSError)?;

        // 1 = data, 0 = command
        self.dc.set_high().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        let err = send_iter(&mut self.spi, iter);

        // Deassert chip select pin
        self.cs.set_high().ok();

        err
    }

    #[inline]
    fn send_command_slice(&mut self, slice: &[Self::Word]) -> Result<(), DisplayError> {
        // Assert chip select pin
        self.cs.set_low().map_err(|_| DisplayError::CSError)?;

        // 1 = data, 0 = command
        self.dc.set_low().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        let err = self
            .spi
            .write(slice)
            .map_err(|_| DisplayError::BusWriteError);

        // Deassert chip select pin
        self.cs.set_high().ok();

        err
    }

    #[inline]
    fn send_data_slice(&mut self, slice: &[Self::Word]) -> Result<(), DisplayError> {
        // Assert chip select pin
        self.cs.set_low().map_err(|_| DisplayError::CSError)?;

        // 1 = data, 0 = command
        self.dc.set_high().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        let err = self
            .spi
            .write(slice)
            .map_err(|_| DisplayError::BusWriteError);

        // Deassert chip select pin
        self.cs.set_high().ok();

        err
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
    SPI: hal::blocking::spi::Write<u8>,
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
    SPI: hal::blocking::spi::Write<u8>,
    DC: OutputPin,
{
    type Word = u8;

    #[inline]
    fn send_command_iter(
        &mut self,
        iter: impl Iterator<Item = Self::Word>,
    ) -> Result<(), DisplayError> {
        // 1 = data, 0 = command
        self.dc.set_low().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        send_iter(&mut self.spi, iter)
    }

    #[inline]
    fn send_data_iter(
        &mut self,
        iter: impl Iterator<Item = Self::Word>,
    ) -> Result<(), DisplayError> {
        // 1 = data, 0 = command
        self.dc.set_high().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        send_iter(&mut self.spi, iter)
    }

    #[inline]
    fn send_command_slice(&mut self, slice: &[Self::Word]) -> Result<(), DisplayError> {
        // 1 = data, 0 = command
        self.dc.set_low().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        self.spi
            .write(slice)
            .map_err(|_| DisplayError::BusWriteError)
    }

    #[inline]
    fn send_data_slice(&mut self, slice: &[Self::Word]) -> Result<(), DisplayError> {
        // 1 = data, 0 = command
        self.dc.set_high().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        self.spi
            .write(slice)
            .map_err(|_| DisplayError::BusWriteError)
    }
}
