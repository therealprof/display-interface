#![no_std]

//! Generic SPI interface for display drivers

use core::marker::PhantomData;

use embedded_hal as hal;
use hal::digital::v2::OutputPin;

use display_interface::{DisplayError, WriteOnlyDataCommand};

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
}

impl<SPI, DC, CS, WIDTH> WriteOnlyDataCommand<WIDTH> for SPIInterface<SPI, DC, CS>
where
    SPI: hal::blocking::spi::Write<WIDTH>,
    DC: OutputPin,
    CS: OutputPin,
{
    fn send_commands(&mut self, cmds: &[WIDTH]) -> Result<(), DisplayError> {
        self.cs.set_low().map_err(|_| DisplayError::CSError)?;
        // 1 = data, 0 = command
        self.dc.set_low().map_err(|_| DisplayError::DCError)?;
        let err = self
            .spi
            .write(&cmds)
            .map_err(|_| DisplayError::BusWriteError);
        self.cs.set_high().ok();
        err
    }

    fn send_data(&mut self, buf: &[WIDTH]) -> Result<(), DisplayError> {
        self.cs.set_low().map_err(|_| DisplayError::CSError)?;
        // 1 = data, 0 = command
        self.dc.set_high().map_err(|_| DisplayError::DCError)?;
        let err = self
            .spi
            .write(&buf)
            .map_err(|_| DisplayError::BusWriteError);
        self.cs.set_high().ok();
        err
    }
}

/// SPI display interface.
///
/// This combines the SPI peripheral and a data/command pin
pub struct SPIInterfaceNoCS<SPI, DC, WIDTH> {
    spi: SPI,
    dc: DC,
    _width: PhantomData<WIDTH>,
}

impl<SPI, DC, WIDTH> SPIInterfaceNoCS<SPI, DC, WIDTH>
where
    SPI: hal::blocking::spi::Write<WIDTH>,
    DC: OutputPin,
{
    /// Create new SPI interface for communciation with a display driver
    pub fn new(spi: SPI, dc: DC) -> Self {
        Self {
            spi,
            dc,
            _width: PhantomData,
        }
    }
}

impl<SPI, DC, WIDTH> WriteOnlyDataCommand<WIDTH> for SPIInterfaceNoCS<SPI, DC, WIDTH>
where
    SPI: hal::blocking::spi::Write<WIDTH>,
    DC: OutputPin,
{
    fn send_commands(&mut self, cmds: &[WIDTH]) -> Result<(), DisplayError> {
        // 1 = data, 0 = command
        self.dc.set_low().map_err(|_| DisplayError::DCError)?;
        self.spi
            .write(&cmds)
            .map_err(|_| DisplayError::BusWriteError)
    }

    fn send_data(&mut self, buf: &[WIDTH]) -> Result<(), DisplayError> {
        // 1 = data, 0 = command
        self.dc.set_high().map_err(|_| DisplayError::DCError)?;
        self.spi
            .write(&buf)
            .map_err(|_| DisplayError::BusWriteError)
    }
}
