#![no_std]

//! Generic parallel GPIO interface for display drivers

use embedded_hal::digital::v2::OutputPin;

pub use display_interface::{DisplayError, WriteOnlyDataCommand};

/// Parallel 8 Bit communication interface trait.
///
/// This interface implements an 8-Bit "8080" style write-only display interface using any
/// `embedded_hal` `digital::v2::OutputPin` implementation.
///
/// To get an actual struct type implementing `PGPIO8BitInterface`, use the macro
/// `pgpio8bit_interface!` in your code.
///
/// For the 8-Bit implementation you need to provide 8 types implementing `OutputPin` which
/// ressemble the bits 0 through 7 (which bit 0 being the LSB and 7 the MSB) as well as one
/// `OutputPin` for the data/command selection and one `OutputPin` for the write-enable flag.
///
/// All pins are supposed to be high-active, high for the D/C pin meaning "data" and the
/// write-enable being pulled low before the setting of the bits and supposed to be sampled at a
/// low to high edge.
pub trait PGPIO8BitInterface {
    type P0: OutputPin;
    type P1: OutputPin;
    type P2: OutputPin;
    type P3: OutputPin;
    type P4: OutputPin;
    type P5: OutputPin;
    type P6: OutputPin;
    type P7: OutputPin;
    type DC: OutputPin;
    type WR: OutputPin;

    fn p0(&mut self) -> &mut Self::P0;
    fn p1(&mut self) -> &mut Self::P1;
    fn p2(&mut self) -> &mut Self::P2;
    fn p3(&mut self) -> &mut Self::P3;
    fn p4(&mut self) -> &mut Self::P4;
    fn p5(&mut self) -> &mut Self::P5;
    fn p6(&mut self) -> &mut Self::P6;
    fn p7(&mut self) -> &mut Self::P7;
    fn dc(&mut self) -> &mut Self::DC;
    fn wr(&mut self) -> &mut Self::WR;
    fn last(&mut self) -> &mut u8;

    fn set_value(self: &mut Self, value: u8) -> Result<(), DisplayError> {
        let changed = value ^ *self.last();
        *self.last() = value;

        if changed & 1 != 0 {
            if value & 1 != 0 {
                self.p0().set_high()
            } else {
                self.p0().set_low()
            }
            .map_err(|_| DisplayError::BusWriteError)?
        };

        if changed & 2 != 0 {
            if value & 2 != 0 {
                self.p1().set_high()
            } else {
                self.p1().set_low()
            }
            .map_err(|_| DisplayError::BusWriteError)?
        };

        if changed & 4 != 0 {
            if value & 4 != 0 {
                self.p2().set_high()
            } else {
                self.p2().set_low()
            }
            .map_err(|_| DisplayError::BusWriteError)?
        };

        if changed & 8 != 0 {
            if value & 8 != 0 {
                self.p3().set_high()
            } else {
                self.p3().set_low()
            }
            .map_err(|_| DisplayError::BusWriteError)?
        };

        if changed & 16 != 0 {
            if value & 16 != 0 {
                self.p4().set_high()
            } else {
                self.p4().set_low()
            }
            .map_err(|_| DisplayError::BusWriteError)?
        };

        if changed & 32 != 0 {
            if value & 32 != 0 {
                self.p5().set_high()
            } else {
                self.p5().set_low()
            }
            .map_err(|_| DisplayError::BusWriteError)?
        };

        if changed & 64 != 0 {
            if value & 64 != 0 {
                self.p6().set_high()
            } else {
                self.p6().set_low()
            }
            .map_err(|_| DisplayError::BusWriteError)?
        };

        if changed & 128 != 0 {
            if value & 128 != 0 {
                self.p7().set_high()
            } else {
                self.p7().set_low()
            }
            .map_err(|_| DisplayError::BusWriteError)?
        };

        Ok(())
    }
}

/// Macro to generate a specific (non-generic) struct type which implements
/// `WriteOnlyDataCommand<u8>` (and `PGPIO8BitInterface`).
///
/// An example call might be:
/// ```ignore
/// use display_interface_parallel_gpio::pgpio8bit_interface;
///
/// pgpio8bit_interface!(
///     MyPGPIO8BitInterface,
///     gpioa::PA8<gpio::Output<gpio::PushPull>>,
///     gpioa::PA9<gpio::Output<gpio::PushPull>>,
///     gpioa::PA10<gpio::Output<gpio::PushPull>>,
///     gpioa::PA11<gpio::Output<gpio::PushPull>>,
///     gpioa::PA12<gpio::Output<gpio::PushPull>>,
///     gpioa::PA6<gpio::Output<gpio::PushPull>>,
///     gpioa::PA7<gpio::Output<gpio::PushPull>>,
///     gpioa::PA15<gpio::Output<gpio::PushPull>>,
///     gpiob::PB7<gpio::Output<gpio::PushPull>>,
///     gpioc::PC14<gpio::Output<gpio::PushPull>>,
/// );
/// ```
///
/// The resulting type `MyPGPIO8BitInterface` could then, for instance, be placed in a
/// ```ignore
/// static INTF: Mutex<RefCell<Option<MyPGPIO8BitInterface>>> = Mutex::new(RefCell::new(None));
/// let interface = MyPGPIO8BitInterface::new(p0, p1, p2, p3, p4, p5, p6, p7, dc, wr);
/// *INTF.borrow(cs).borrow_mut() = Some(interface);
/// ```
/// without repeating the input pin types.
///
/// Or it could be used as a building block in a type that bounds on
/// `WriteOnlyDataCommand<u8> + PGPIO8BitInterface`.
#[macro_export]
macro_rules! pgpio8bit_interface {
    (
        $Name:ident,
        $P0:ty,
        $P1:ty,
        $P2:ty,
        $P3:ty,
        $P4:ty,
        $P5:ty,
        $P6:ty,
        $P7:ty,
        $DC:ty,
        $WR:ty,
    ) => {
        pub struct $Name {
            p0: $P0,
            p1: $P1,
            p2: $P2,
            p3: $P3,
            p4: $P4,
            p5: $P5,
            p6: $P6,
            p7: $P7,
            dc: $DC,
            wr: $WR,

            last: u8,
        }

        impl $Name {
            /// Create new parallel GPIO interface for communication with a display driver
            #[allow(clippy::too_many_arguments)]
            pub fn new(
                p0: $P0,
                p1: $P1,
                p2: $P2,
                p3: $P3,
                p4: $P4,
                p5: $P5,
                p6: $P6,
                p7: $P7,
                dc: $DC,
                wr: $WR,
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
        }

        impl $crate::PGPIO8BitInterface for $Name {
            type P0 = $P0;
            type P1 = $P1;
            type P2 = $P2;
            type P3 = $P3;
            type P4 = $P4;
            type P5 = $P5;
            type P6 = $P6;
            type P7 = $P7;
            type DC = $DC;
            type WR = $WR;

            fn p0(&mut self) -> &mut Self::P0 { &mut self.p0 }
            fn p1(&mut self) -> &mut Self::P1 { &mut self.p1 }
            fn p2(&mut self) -> &mut Self::P2 { &mut self.p2 }
            fn p3(&mut self) -> &mut Self::P3 { &mut self.p3 }
            fn p4(&mut self) -> &mut Self::P4 { &mut self.p4 }
            fn p5(&mut self) -> &mut Self::P5 { &mut self.p5 }
            fn p6(&mut self) -> &mut Self::P6 { &mut self.p6 }
            fn p7(&mut self) -> &mut Self::P7 { &mut self.p7 }
            fn dc(&mut self) -> &mut Self::DC { &mut self.dc }
            fn wr(&mut self) -> &mut Self::WR { &mut self.wr }
            fn last(&mut self) -> &mut u8 { &mut self.last }
        }

        impl $crate::WriteOnlyDataCommand<u8> for $Name {
            fn send_commands(&mut self, cmds: &[u8]) -> Result<(), $crate::DisplayError> {
                $crate::send_commands(self, cmds)
            }
            fn send_data(&mut self, buf: &[u8]) -> Result<(), $crate::DisplayError> {
                $crate::send_data(self, buf)
            }
        }
    }
}

// Module-level functions to keep the macro itself minimal.

pub fn send_commands(interface: &mut impl PGPIO8BitInterface, cmds: &[u8]) -> Result<(), DisplayError> {
    interface.dc().set_low().map_err(|_| DisplayError::BusWriteError)?;
    cmds.iter().try_for_each(|cmd| {
        interface.wr().set_low().map_err(|_| DisplayError::BusWriteError)?;
        interface.set_value(*cmd)?;
        interface.wr().set_high().map_err(|_| DisplayError::BusWriteError)
    })
}

pub fn send_data(interface: &mut impl PGPIO8BitInterface, buf: &[u8]) -> Result<(), DisplayError> {
    interface.dc()
        .set_high()
        .map_err(|_| DisplayError::BusWriteError)?;
    buf.iter().try_for_each(|d| {
        interface.wr().set_low().map_err(|_| DisplayError::BusWriteError)?;
        interface.set_value(*d)?;
        interface.wr().set_high().map_err(|_| DisplayError::BusWriteError)
    })
}
