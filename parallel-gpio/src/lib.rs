#![no_std]

//! Generic parallel GPIO interface for display drivers

use embedded_hal::digital::v2::OutputPin;

pub use display_interface::{DisplayError, WriteOnlyDataCommand};

/// This trait represents the data pins of a parallel bus.
///
/// See [Generic8BitBus] for a generic implementation.
pub trait OutputBus {
    /// [u8] for 8-bit busses, [u16] for 16-bit busses, etc.
    type Word: Copy;

    fn set_value(&mut self, value: Self::Word) -> Result<(), DisplayError>;
}

macro_rules! generic_bus {
    ($GenericxBitBus:ident { type Word = $Word:ident; Pins {$($PX:ident => $x:tt,)*}}) => {
        /// A generic implementation of [OutputBus] using [OutputPin]s
        pub struct $GenericxBitBus<$($PX, )*> {
            pins: ($($PX, )*),
            last: $Word,
        }

        impl<$($PX, )*> $GenericxBitBus<$($PX, )*>
        where
            $($PX: OutputPin, )*
        {
            /// Creates a new instance and initializes the bus to `0`.
            ///
            /// The first pin in the tuple is the least significant bit.
            pub fn new(pins: ($($PX, )*)) -> Result<Self, DisplayError> {
                let mut bus = Self { pins, last: $Word::MAX };

                // By setting `last` to all ones, we ensure that this will update all the pins
                bus.set_value(0)?;

                Ok(bus)
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

            fn set_value(&mut self, value: Self::Word) -> Result<(), DisplayError> {
                let changed = value ^ self.last;

                // It's quite common for multiple consecutive values to be identical, e.g. when filling or
                // clearing the screen, so let's optimize for that case
                if changed != 0 {
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

                    self.last = value;
                }

                Ok(())
            }
        }

        impl<$($PX, )*> core::convert::TryFrom<($($PX, )*)>
            for $GenericxBitBus<$($PX, )*>
        where
            $($PX: OutputPin, )*
        {
            type Error = DisplayError;

            fn try_from(pins: ($($PX, )*)) -> Result<Self, Self::Error> {
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

/// Parallel communication interface
///
/// This interface implements an "8080" style write-only display interface using any
/// [OutputBus] implementation as well as one
/// `OutputPin` for the data/command selection and one `OutputPin` for the write-enable flag.
///
/// All pins are supposed to be high-active, high for the D/C pin meaning "data" and the
/// write-enable being pulled low before the setting of the bits and supposed to be sampled at a
/// low to high edge.
pub struct ParallelInterface<BUS, DC, WR> {
    bus: BUS,
    dc: DC,
    wr: WR,
}

impl<BUS, DC, WR> ParallelInterface<BUS, DC, WR>
where
    BUS: OutputBus,
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

    fn set_value(self: &mut Self, value: BUS::Word) -> Result<(), DisplayError> {
        self.bus.set_value(value)
    }

    fn send_iter<I: Iterator<Item = BUS::Word>>(&mut self, mut iter: I) -> Result<(), DisplayError> {
        iter.try_for_each(|d| {
            self.wr.set_low().map_err(|_| DisplayError::BusWriteError)?;
            self.set_value(d)?;
            self.wr.set_high().map_err(|_| DisplayError::BusWriteError)
        })
    }
}

impl<BUS, DC, WR> WriteOnlyDataCommand for ParallelInterface<BUS, DC, WR>
where
    BUS: OutputBus,
    DC: OutputPin,
    WR: OutputPin,
{
    type Word = BUS::Word;

    fn send_command_iter(
        &mut self,
        iter: impl Iterator<Item = Self::Word>,
    ) -> Result<(), DisplayError> {
        self.dc.set_low().map_err(|_| DisplayError::DCError)?;

        self.send_iter(iter)
    }

    fn send_data_iter(
        &mut self,
        iter: impl Iterator<Item = Self::Word>,
    ) -> Result<(), DisplayError> {
        self.dc.set_high().map_err(|_| DisplayError::DCError)?;

        self.send_iter(iter)
    }
}
