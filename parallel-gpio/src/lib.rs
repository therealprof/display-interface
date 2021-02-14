#![no_std]

//! Generic parallel GPIO interface for display drivers

use embedded_hal::digital::v2::OutputPin;

pub use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};

/// Parallel 8 Bit communication interface
///
/// This interface implements an 8-Bit "8080" style write-only display interface using any
/// `embedded_hal` `digital::v2::OutputPin` implementation.
///
/// For the 8-Bit implementation you need to provide 8 types implementing `OutputPin` which
/// ressemble the bits 0 through 7 (which bit 0 being the LSB and 7 the MSB) as well as one
/// `OutputPin` for the data/command selection and one `OutputPin` for the write-enable flag.
///
/// All pins are supposed to be high-active, high for the D/C pin meaning "data" and the
/// write-enable being pulled low before the setting of the bits and supposed to be sampled at a
/// low to high edge.
pub struct PGPIO8BitInterface<DBUS, DC, WR> {
    dbus: [DBUS; 8],
    dc: DC,
    wr: WR,
    last: u8,
}

impl<DBUS, DC, WR> PGPIO8BitInterface<DBUS, DC, WR>
where
    DBUS: OutputPin,
    DC: OutputPin,
    WR: OutputPin,
{
    /// Create new parallel GPIO interface for communication with a display driver
    pub fn new(dbus: [DBUS; 8], dc: DC, wr: WR) -> Self {
        Self {
            dbus,
            dc,
            wr,
            last: 0,
        }
    }

    /// Consume the display interface and return
    /// the GPIO pins used by it
    pub fn release(self) -> ([DBUS; 8], DC, WR) {
        (self.dbus, self.dc, self.wr)
    }

    fn set_value(self: &mut Self, value: u8) -> Result<(), DisplayError> {
        let changed = value ^ self.last;

        // It's quite common for multiple consecutive values to be identical, e.g. when filling or
        // clearing the screen, so let's optimize for that case
        if changed == 0 {
            return Ok(());
        }

        self.last = value;

        for i in 0..8 {
            if changed & (1 << i) != 0 {
                if value & (1 << i) == 0 {
                    self.dbus[i].set_low()
                } else {
                    self.dbus[i].set_high()
                }
                .map_err(|_| DisplayError::BusWriteError)?;
            }
        }

        Ok(())
    }
}

impl<DBUS, DC, WR> WriteOnlyDataCommand for PGPIO8BitInterface<DBUS, DC, WR>
where
    DBUS: OutputPin,
    DC: OutputPin,
    WR: OutputPin,
{
    fn send_commands(&mut self, cmds: DataFormat<'_>) -> Result<(), DisplayError> {
        use byte_slice_cast::*;
        self.dc.set_low().map_err(|_| DisplayError::BusWriteError)?;
        match cmds {
            DataFormat::U8(slice) => slice.iter().try_for_each(|cmd| {
                self.wr.set_low().map_err(|_| DisplayError::BusWriteError)?;
                self.set_value(*cmd)?;
                self.wr.set_high().map_err(|_| DisplayError::BusWriteError)
            }),
            DataFormat::U8Iter(iter) => {
                for c in iter.into_iter() {
                    self.wr.set_low().map_err(|_| DisplayError::BusWriteError)?;
                    self.set_value(c)?;
                    self.wr
                        .set_high()
                        .map_err(|_| DisplayError::BusWriteError)?;
                }

                Ok(())
            }
            DataFormat::U16(slice) => slice.as_byte_slice().iter().try_for_each(|cmd| {
                self.wr.set_low().map_err(|_| DisplayError::BusWriteError)?;
                self.set_value(*cmd)?;
                self.wr.set_high().map_err(|_| DisplayError::BusWriteError)
            }),
            DataFormat::U16LE(slice) => slice.iter().try_for_each(|cmd| {
                for cmd in &cmd.to_le_bytes() {
                    self.wr.set_low().map_err(|_| DisplayError::BusWriteError)?;
                    self.set_value(*cmd)?;
                    self.wr
                        .set_high()
                        .map_err(|_| DisplayError::BusWriteError)?;
                }

                Ok(())
            }),
            DataFormat::U16BE(slice) => slice.iter().try_for_each(|cmd| {
                for cmd in &cmd.to_be_bytes() {
                    self.wr.set_low().map_err(|_| DisplayError::BusWriteError)?;
                    self.set_value(*cmd)?;
                    self.wr
                        .set_high()
                        .map_err(|_| DisplayError::BusWriteError)?;
                }

                Ok(())
            }),
            _ => Err(DisplayError::DataFormatNotImplemented),
        }
    }

    fn send_data(&mut self, buf: DataFormat<'_>) -> Result<(), DisplayError> {
        use byte_slice_cast::*;
        self.dc
            .set_high()
            .map_err(|_| DisplayError::BusWriteError)?;
        match buf {
            DataFormat::U8(slice) => slice.iter().try_for_each(|d| {
                self.wr.set_low().map_err(|_| DisplayError::BusWriteError)?;
                self.set_value(*d)?;
                self.wr.set_high().map_err(|_| DisplayError::BusWriteError)
            }),
            DataFormat::U16(slice) => slice.as_byte_slice().iter().try_for_each(|d| {
                self.wr.set_low().map_err(|_| DisplayError::BusWriteError)?;
                self.set_value(*d)?;
                self.wr.set_high().map_err(|_| DisplayError::BusWriteError)
            }),
            DataFormat::U16LE(slice) => slice.iter().try_for_each(|cmd| {
                for cmd in &cmd.to_le_bytes() {
                    self.wr.set_low().map_err(|_| DisplayError::BusWriteError)?;
                    self.set_value(*cmd)?;
                    self.wr
                        .set_high()
                        .map_err(|_| DisplayError::BusWriteError)?;
                }

                Ok(())
            }),
            DataFormat::U16BE(slice) => slice.iter().try_for_each(|cmd| {
                for cmd in &cmd.to_be_bytes() {
                    self.wr.set_low().map_err(|_| DisplayError::BusWriteError)?;
                    self.set_value(*cmd)?;
                    self.wr
                        .set_high()
                        .map_err(|_| DisplayError::BusWriteError)?;
                }

                Ok(())
            }),
            DataFormat::U8Iter(iter) => {
                for d in iter.into_iter() {
                    self.wr.set_low().map_err(|_| DisplayError::BusWriteError)?;
                    self.set_value(d)?;
                    self.wr
                        .set_high()
                        .map_err(|_| DisplayError::BusWriteError)?;
                }

                Ok(())
            }
            DataFormat::U16LEIter(iter) => {
                for cmd in iter.into_iter() {
                    for cmd in &cmd.to_le_bytes() {
                        self.wr.set_low().map_err(|_| DisplayError::BusWriteError)?;
                        self.set_value(*cmd)?;
                        self.wr
                            .set_high()
                            .map_err(|_| DisplayError::BusWriteError)?;
                    }
                }
                Ok(())
            }
            DataFormat::U16BEIter(iter) => {
                for cmd in iter.into_iter() {
                    for cmd in &cmd.to_be_bytes() {
                        self.wr.set_low().map_err(|_| DisplayError::BusWriteError)?;
                        self.set_value(*cmd)?;
                        self.wr
                            .set_high()
                            .map_err(|_| DisplayError::BusWriteError)?;
                    }
                }
                Ok(())
            }
            _ => Err(DisplayError::DataFormatNotImplemented),
        }
    }
}
