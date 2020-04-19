#![no_main]
#![no_std]

use panic_halt as _;

use stm32f0xx_hal as hal;

use hal::{
    delay::Delay,
    gpio::{self, gpioa, gpiob, gpioc},
    pac,
    pac::{interrupt, Interrupt, TIM7},
    prelude::*,
    serial::Serial,
    timers::{Event, Timer},
};

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_graphics::style::*;

use st7789::{Orientation, ST7789};

use core::cell::RefCell;
use core::fmt::Write as _;
use core::ops::DerefMut;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;

use display_interface_parallel_gpio::pgpio8bit_interface;

pgpio8bit_interface!(
    PGPIO8BitInterface,
    gpioa::PA8<gpio::Output<gpio::PushPull>>,
    gpioa::PA9<gpio::Output<gpio::PushPull>>,
    gpioa::PA10<gpio::Output<gpio::PushPull>>,
    gpioa::PA11<gpio::Output<gpio::PushPull>>,
    gpioa::PA12<gpio::Output<gpio::PushPull>>,
    gpioa::PA6<gpio::Output<gpio::PushPull>>,
    gpioa::PA7<gpio::Output<gpio::PushPull>>,
    gpioa::PA15<gpio::Output<gpio::PushPull>>,
    gpiob::PB7<gpio::Output<gpio::PushPull>>,
    gpioc::PC14<gpio::Output<gpio::PushPull>>,
);

// Make timer interrupt registers globally available
static GINT: Mutex<RefCell<Option<Timer<TIM7>>>> = Mutex::new(RefCell::new(None));

#[derive(Copy, Clone)]
struct Time {
    seconds: u32,
    millis: u16,
}

static TIME: Mutex<RefCell<Time>> = Mutex::new(RefCell::new(Time {
    seconds: 0,
    millis: 0,
}));

// Define an interupt handler, i.e. function to call when interrupt occurs. Here if our external
// interrupt trips when the timer timed out
#[interrupt]
fn TIM7() {
    cortex_m::interrupt::free(|cs| {
        // Move LED pin here, leaving a None in its place
        GINT.borrow(cs)
            .borrow_mut()
            .deref_mut()
            .as_mut()
            .unwrap()
            .wait()
            .ok();
        let mut time = TIME.borrow(cs).borrow_mut();
        time.millis += 1;
        if time.millis == 1000 {
            time.millis = 0;
            time.seconds += 1;
        }
    });
}


#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (pac::Peripherals::take(), cortex_m::Peripherals::take()) {
        let (mut serial, mut display) = cortex_m::interrupt::free(move |cs| {
            let mut flash = p.FLASH;
            let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut flash);

            // Use USART2 with PA2 and PA3 as serial port
            let gpioa = p.GPIOA.split(&mut rcc);
            let tx = gpioa.pa2.into_alternate_af1(cs);
            let rx = gpioa.pa3.into_alternate_af1(cs);

            let gpiob = p.GPIOB.split(&mut rcc);
            let gpioc = p.GPIOC.split(&mut rcc);

            // Initialise delay provider
            let delay = Delay::new(cp.SYST, &rcc);

            // Configure aux pins for display
            let dc = gpiob.pb7.into_push_pull_output(cs);
            let rst = gpioc.pc15.into_push_pull_output(cs);

            // Configure main pins for parallel display interface
            let p0 = gpioa.pa8.into_push_pull_output(cs);
            let p1 = gpioa.pa9.into_push_pull_output(cs);
            let p2 = gpioa.pa10.into_push_pull_output(cs);
            let p3 = gpioa.pa11.into_push_pull_output(cs);
            let p4 = gpioa.pa12.into_push_pull_output(cs);
            let p5 = gpioa.pa6.into_push_pull_output(cs);
            let p6 = gpioa.pa7.into_push_pull_output(cs);
            let p7 = gpioa.pa15.into_push_pull_output(cs);
            let wr = gpioc.pc14.into_push_pull_output(cs);

            // Set up a timer expiring every millisecond
            let mut timer = Timer::tim7(p.TIM7, 1000.hz(), &mut rcc);

            // Generate an interrupt when the timer expires
            timer.listen(Event::TimeOut);

            // Move the timer into our global storage
            *GINT.borrow(cs).borrow_mut() = Some(timer);

            // Set up our serial port
            let serial = Serial::usart2(p.USART2, (tx, rx), 115_200.bps(), &mut rcc);

            // create driver
            let interface = PGPIO8BitInterface::new(p0, p1, p2, p3, p4, p5, p6, p7, dc, wr);
            let mut display = ST7789::new(interface, rst, 240, 240, delay);

            // initialize
            display.init().unwrap();
            // set default orientation
            display.set_orientation(&Orientation::Landscape).unwrap();

            // Enable TIM7 IRQ, set prio 1 and clear any pending IRQs
            let mut nvic = cp.NVIC;
            unsafe {
                nvic.set_priority(Interrupt::TIM7, 1);
                cortex_m::peripheral::NVIC::unmask(Interrupt::TIM7);
            }
            cortex_m::peripheral::NVIC::unpend(Interrupt::TIM7);

            (serial, display)
        });

        let circle1 = Circle::new(Point::new(128, 64), 64)
            .into_styled(PrimitiveStyle::with_fill(Rgb565::RED));
        let circle2 = Circle::new(Point::new(64, 64), 64)
            .into_styled(PrimitiveStyle::with_stroke(Rgb565::GREEN, 1));

        let blue_with_red_outline = PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::BLUE)
            .stroke_color(Rgb565::RED)
            .stroke_width(1) // > 1 is not currently suppored in embedded-graphics on triangles
            .build();
        let triangle = Triangle::new(
            Point::new(40, 120),
            Point::new(40, 220),
            Point::new(140, 120),
        )
        .into_styled(blue_with_red_outline);

        let line = Line::new(Point::new(180, 160), Point::new(239, 239))
            .into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, 10));

        loop {
            //blank.draw(&mut display).unwrap();
            display.clear(Rgb565::BLACK).unwrap();

            cortex_m::interrupt::free(|cs| {
                let mut time = TIME.borrow(cs).borrow_mut();

                // Print the current time
                writeln!(serial, "blank: {}.{:03}s\r", time.seconds, time.millis).ok();

                // Reset the time
                time.millis = 0;
                time.seconds = 0;
            });

            // draw two circles on blue background
            circle1.draw(&mut display).unwrap();
            circle2.draw(&mut display).unwrap();
            triangle.draw(&mut display).unwrap();
            line.draw(&mut display).unwrap();

            cortex_m::interrupt::free(|cs| {
                let mut time = TIME.borrow(cs).borrow_mut();

                // Print the current time
                writeln!(serial, "{}.{:03}s\r", time.seconds, time.millis).ok();

                // Reset the time
                time.millis = 0;
                time.seconds = 0;
            });
        }
    }

    loop {
        continue;
    }
}
