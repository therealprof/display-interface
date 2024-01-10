#![no_std]

//! Generic parallel GPIO interface for display drivers

use embedded_hal::digital::OutputPin;

pub use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};

type Result<T = ()> = core::result::Result<T, DisplayError>;

/// This trait represents the data pins of a parallel bus.
///
/// See [Generic8BitBus] and [Generic16BitBus] for generic implementations.
pub trait OutputBus {
    /// [u8] for 8-bit buses, [u16] for 16-bit buses, etc.
    type Word: Copy;

    fn set_value(&mut self, value: Self::Word) -> Result;
}

macro_rules! generic_bus {
    ($GenericxBitBus:ident { type Word = $Word:ident; Pins {$($PX:ident => $x:tt,)*}}) => {
        /// A generic implementation of [OutputBus] using [OutputPin]s
        pub struct $GenericxBitBus<$($PX, )*> {
            pins: ($($PX, )*),
            last: Option<$Word>,
        }

        impl<$($PX, )*> $GenericxBitBus<$($PX, )*>
        where
            $($PX: OutputPin, )*
        {
            /// Creates a new bus. This does not change the state of the pins.
            ///
            /// The first pin in the tuple is the least significant bit.
            pub fn new(pins: ($($PX, )*)) -> Self {
                Self { pins, last: None }
            }

            /// Consumes the bus and returns the pins. This does not change the state of the pins.
            pub fn release(self) -> ($($PX, )*) {
                self.pins
            }
        }

        impl<$($PX, )*> OutputBus
            for $GenericxBitBus<$($PX, )*>
        where
            $($PX: OutputPin, )*
        {
            type Word = $Word;

            fn set_value(&mut self, value: Self::Word) -> Result {
                if self.last == Some(value) {
                    // It's quite common for multiple consecutive values to be identical, e.g. when filling or
                    // clearing the screen, so let's optimize for that case
                    return Ok(())
                }

                // Sets self.last to None.
                // We will update it to Some(value) *after* all the pins are succesfully set.
                let last = self.last.take();

                let changed = match last {
                    Some(old_value) => value ^ old_value,
                    None => !0, // all ones, this ensures that we will update all the pins
                };

                $(
                    let mask = 1 << $x;
                    if changed & mask != 0 {
                        if value & mask != 0 {
                            self.pins.$x.set_high()
                        } else {
                            self.pins.$x.set_low()
                        }
                        .map_err(|_| DisplayError::BusWriteError)?;
                    }
                )*

                self.last = Some(value);
                Ok(())
            }
        }

        impl<$($PX, )*> From<($($PX, )*)>
            for $GenericxBitBus<$($PX, )*>
        where
            $($PX: OutputPin, )*
        {
            fn from(pins: ($($PX, )*)) -> Self {
                Self::new(pins)
            }
        }
    };
}

generic_bus! {
    Generic8BitBus {
        type Word = u8;
        Pins {
            P0 => 0,
            P1 => 1,
            P2 => 2,
            P3 => 3,
            P4 => 4,
            P5 => 5,
            P6 => 6,
            P7 => 7,
        }
    }
}

generic_bus! {
    Generic16BitBus {
        type Word = u16;
        Pins {
            P0 => 0,
            P1 => 1,
            P2 => 2,
            P3 => 3,
            P4 => 4,
            P5 => 5,
            P6 => 6,
            P7 => 7,
            P8 => 8,
            P9 => 9,
            P10 => 10,
            P11 => 11,
            P12 => 12,
            P13 => 13,
            P14 => 14,
            P15 => 15,
        }
    }
}

/// Parallel 8 Bit communication interface
///
/// This interface implements an 8-Bit "8080" style write-only display interface using any
/// 8-bit [OutputBus] implementation as well as one
/// `OutputPin` for the data/command selection and one `OutputPin` for the write-enable flag.
///
/// All pins are supposed to be high-active, high for the D/C pin meaning "data" and the
/// write-enable being pulled low before the setting of the bits and supposed to be sampled at a
/// low to high edge.
pub struct PGPIO8BitInterface<BUS, DC, WR> {
    bus: BUS,
    dc: DC,
    wr: WR,
}

impl<BUS, DC, WR> PGPIO8BitInterface<BUS, DC, WR>
where
    BUS: OutputBus<Word = u8>,
    DC: OutputPin,
    WR: OutputPin,
{
    /// Create new parallel GPIO interface for communication with a display driver
    pub fn new(bus: BUS, dc: DC, wr: WR) -> Self {
        Self { bus, dc, wr }
    }

    /// Consume the display interface and return
    /// the bus and GPIO pins used by it
    pub fn release(self) -> (BUS, DC, WR) {
        (self.bus, self.dc, self.wr)
    }

    fn write_iter(&mut self, iter: impl Iterator<Item = u8>) -> Result {
        for value in iter {
            self.wr.set_low().map_err(|_| DisplayError::BusWriteError)?;
            self.bus.set_value(value)?;
            self.wr
                .set_high()
                .map_err(|_| DisplayError::BusWriteError)?;
        }

        Ok(())
    }

    fn write_pairs(&mut self, iter: impl Iterator<Item = [u8; 2]>) -> Result {
        use core::iter::once;
        self.write_iter(iter.flat_map(|[first, second]| once(first).chain(once(second))))
    }

    fn write_data(&mut self, data: DataFormat<'_>) -> Result {
        match data {
            DataFormat::U8(slice) => self.write_iter(slice.iter().copied()),
            DataFormat::U8Iter(iter) => self.write_iter(iter),
            DataFormat::U16(slice) => self.write_pairs(slice.iter().copied().map(u16::to_ne_bytes)),
            DataFormat::U16BE(slice) => {
                self.write_pairs(slice.iter().copied().map(u16::to_be_bytes))
            }
            DataFormat::U16LE(slice) => {
                self.write_pairs(slice.iter().copied().map(u16::to_le_bytes))
            }
            DataFormat::U16BEIter(iter) => self.write_pairs(iter.map(u16::to_be_bytes)),
            DataFormat::U16LEIter(iter) => self.write_pairs(iter.map(u16::to_le_bytes)),
            _ => Err(DisplayError::DataFormatNotImplemented),
        }
    }
}

impl<BUS, DC, WR> WriteOnlyDataCommand for PGPIO8BitInterface<BUS, DC, WR>
where
    BUS: OutputBus<Word = u8>,
    DC: OutputPin,
    WR: OutputPin,
{
    fn send_commands(&mut self, cmds: DataFormat<'_>) -> Result {
        self.dc.set_low().map_err(|_| DisplayError::DCError)?;
        self.write_data(cmds)
    }

    fn send_data(&mut self, buf: DataFormat<'_>) -> Result {
        self.dc.set_high().map_err(|_| DisplayError::DCError)?;
        self.write_data(buf)
    }
}

/// Parallel 16 Bit communication interface
///
/// This interface implements a 16-Bit "8080" style write-only display interface using any
/// 16-bit [OutputBus] implementation as well as one
/// `OutputPin` for the data/command selection and one `OutputPin` for the write-enable flag.
///
/// All pins are supposed to be high-active, high for the D/C pin meaning "data" and the
/// write-enable being pulled low before the setting of the bits and supposed to be sampled at a
/// low to high edge.
pub struct PGPIO16BitInterface<BUS, DC, WR> {
    bus: BUS,
    dc: DC,
    wr: WR,
}

impl<BUS, DC, WR> PGPIO16BitInterface<BUS, DC, WR>
where
    BUS: OutputBus<Word = u16>,
    DC: OutputPin,
    WR: OutputPin,
{
    /// Create new parallel GPIO interface for communication with a display driver
    pub fn new(bus: BUS, dc: DC, wr: WR) -> Self {
        Self { bus, dc, wr }
    }

    /// Consume the display interface and return
    /// the bus and GPIO pins used by it
    pub fn release(self) -> (BUS, DC, WR) {
        (self.bus, self.dc, self.wr)
    }

    fn write_iter(&mut self, iter: impl Iterator<Item = u16>) -> Result {
        for value in iter {
            self.wr.set_low().map_err(|_| DisplayError::BusWriteError)?;
            self.bus.set_value(value)?;
            self.wr
                .set_high()
                .map_err(|_| DisplayError::BusWriteError)?;
        }

        Ok(())
    }

    fn write_data(&mut self, data: DataFormat<'_>) -> Result {
        match data {
            DataFormat::U8(slice) => self.write_iter(slice.iter().copied().map(u16::from)),
            DataFormat::U8Iter(iter) => self.write_iter(iter.map(u16::from)),
            DataFormat::U16(slice) => self.write_iter(slice.iter().copied()),
            DataFormat::U16BE(slice) => self.write_iter(slice.iter().copied()),
            DataFormat::U16LE(slice) => self.write_iter(slice.iter().copied()),
            DataFormat::U16BEIter(iter) => self.write_iter(iter),
            DataFormat::U16LEIter(iter) => self.write_iter(iter),
            _ => Err(DisplayError::DataFormatNotImplemented),
        }
    }
}

impl<BUS, DC, WR> WriteOnlyDataCommand for PGPIO16BitInterface<BUS, DC, WR>
where
    BUS: OutputBus<Word = u16>,
    DC: OutputPin,
    WR: OutputPin,
{
    fn send_commands(&mut self, cmds: DataFormat<'_>) -> Result {
        self.dc.set_low().map_err(|_| DisplayError::DCError)?;
        self.write_data(cmds)
    }

    fn send_data(&mut self, buf: DataFormat<'_>) -> Result {
        self.dc.set_high().map_err(|_| DisplayError::DCError)?;
        self.write_data(buf)
    }
}
