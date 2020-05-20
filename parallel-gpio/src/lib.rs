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
pub struct PGPIO8BitInterface<P0, P1, P2, P3, P4, P5, P6, P7, DC, WR> {
    p0: P0,
    p1: P1,
    p2: P2,
    p3: P3,
    p4: P4,
    p5: P5,
    p6: P6,
    p7: P7,
    dc: DC,
    wr: WR,
    last: u8,
}

impl<P0, P1, P2, P3, P4, P5, P6, P7, DC, WR>
    PGPIO8BitInterface<P0, P1, P2, P3, P4, P5, P6, P7, DC, WR>
where
    P0: OutputPin,
    P1: OutputPin,
    P2: OutputPin,
    P3: OutputPin,
    P4: OutputPin,
    P5: OutputPin,
    P6: OutputPin,
    P7: OutputPin,
    DC: OutputPin,
    WR: OutputPin,
{
    /// Create new parallel GPIO interface for communication with a display driver
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        p0: P0,
        p1: P1,
        p2: P2,
        p3: P3,
        p4: P4,
        p5: P5,
        p6: P6,
        p7: P7,
        dc: DC,
        wr: WR,
    ) -> Self {
        Self {
            p0,
            p1,
            p2,
            p3,
            p4,
            p5,
            p6,
            p7,
            dc,
            wr,
            last: 0,
        }
    }

    /// Consume the display interface and return
    /// the GPIO pins used by it
    pub fn release(self) -> (P0, P1, P2, P3, P4, P5, P6, P7, DC, WR) {
        (
            self.p0, self.p1, self.p2, self.p3, self.p4, self.p5, self.p6, self.p7, self.dc,
            self.wr,
        )
    }

    fn set_value(self: &mut Self, value: u8) -> Result<(), DisplayError> {
        let changed = value ^ self.last;

        // It's quite common for multiple consecutive values to be identical, e.g. when filling or
        // clearing the screen, so let's optimize for that case
        if changed == 0 {
            return Ok(());
        }

        self.last = value;

        if changed & 1 != 0 {
            if value & 1 != 0 {
                self.p0.set_high()
            } else {
                self.p0.set_low()
            }
            .map_err(|_| DisplayError::BusWriteError)?
        };

        if changed & 2 != 0 {
            if value & 2 != 0 {
                self.p1.set_high()
            } else {
                self.p1.set_low()
            }
            .map_err(|_| DisplayError::BusWriteError)?
        };

        if changed & 4 != 0 {
            if value & 4 != 0 {
                self.p2.set_high()
            } else {
                self.p2.set_low()
            }
            .map_err(|_| DisplayError::BusWriteError)?
        };

        if changed & 8 != 0 {
            if value & 8 != 0 {
                self.p3.set_high()
            } else {
                self.p3.set_low()
            }
            .map_err(|_| DisplayError::BusWriteError)?
        };

        if changed & 16 != 0 {
            if value & 16 != 0 {
                self.p4.set_high()
            } else {
                self.p4.set_low()
            }
            .map_err(|_| DisplayError::BusWriteError)?
        };

        if changed & 32 != 0 {
            if value & 32 != 0 {
                self.p5.set_high()
            } else {
                self.p5.set_low()
            }
            .map_err(|_| DisplayError::BusWriteError)?
        };

        if changed & 64 != 0 {
            if value & 64 != 0 {
                self.p6.set_high()
            } else {
                self.p6.set_low()
            }
            .map_err(|_| DisplayError::BusWriteError)?
        };

        if changed & 128 != 0 {
            if value & 128 != 0 {
                self.p7.set_high()
            } else {
                self.p7.set_low()
            }
            .map_err(|_| DisplayError::BusWriteError)?
        };

        Ok(())
    }
}

impl<P0, P1, P2, P3, P4, P5, P6, P7, DC, WR> WriteOnlyDataCommand
    for PGPIO8BitInterface<P0, P1, P2, P3, P4, P5, P6, P7, DC, WR>
where
    P0: OutputPin,
    P1: OutputPin,
    P2: OutputPin,
    P3: OutputPin,
    P4: OutputPin,
    P5: OutputPin,
    P6: OutputPin,
    P7: OutputPin,
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
        }
    }
}
