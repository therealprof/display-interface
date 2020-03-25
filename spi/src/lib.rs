#![no_std]

//! Generic SPI interface for display drivers
use embedded_hal as hal;
use hal::digital::v2::OutputPin;

use display_interface::WriteOnlyDataCommand;

/// SPI display interface.
///
/// This combines the SPI peripheral and a data/command as well as a chip-select pin
pub struct SPIInterface<SPI, DC, CS> {
    spi: SPI,
    dc: DC,
    cs: Option<CS>,
}

impl<SPI, DC, CS> SPIInterface<SPI, DC, CS>
where
    SPI: hal::blocking::spi::Write<u8>,
    DC: OutputPin,
    CS: OutputPin,
{
    /// Create new SPI interface for communication with a display driver
    pub fn new(spi: SPI, dc: DC, cs: Option<CS>) -> Self {
        Self { spi, dc, cs }
    }
}

impl<SPI, DC, CS> WriteOnlyDataCommand for SPIInterface<SPI, DC, CS>
where
    SPI: hal::blocking::spi::Write<u8>,
    DC: OutputPin,
    CS: OutputPin,
{
    type Error = SPI::Error;

    fn send_commands(&mut self, cmds: &[u8]) -> Result<(), Self::Error> {
        self.dc
            .set_low()
            .and_then(|_| Ok(self.cs.as_mut().map(|mut pin| pin.set_low())));
        self.spi.write(&cmds)
    }

    fn send_data(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        // 1 = data, 0 = command
        self.dc
            .set_high()
            .and_then(|_| Ok(self.cs.as_mut().map(|mut pin| pin.set_low())));

        self.spi.write(&buf)
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
}

impl<SPI, DC> WriteOnlyDataCommand for SPIInterfaceNoCS<SPI, DC>
where
    SPI: hal::blocking::spi::Write<u8>,
    DC: OutputPin,
{
    type Error = SPI::Error;

    fn send_commands(&mut self, cmds: &[u8]) -> Result<(), Self::Error> {
        self.dc.set_low();
        self.spi.write(&cmds)
    }

    fn send_data(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        // 1 = data, 0 = command
        self.dc.set_high();
        self.spi.write(&buf)
    }
}
