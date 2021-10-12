//! blinky timer using interrupts on TIM2, adapted from blinky_timer_irq.rs example from
//! stm32f1xx-hal
//!
//! This assumes that a LED is connected to pa5 (sck/d13) as is the case on most nucleo board.

#![no_main]
#![no_std]

use panic_halt as _;

use stm32f2xx_hal as hal;

use crate::hal::{
    gpio::{gpiob, Output, PushPull},
    pac::{interrupt, Interrupt, Peripherals, TIM2},
    prelude::*,
    timer::{CountDownTimer, Event, Timer},
};

use core::cell::RefCell;
use cortex_m::{asm::wfi, interrupt::Mutex};
use cortex_m_rt::entry;
use embedded_hal::timer::CountDown;

// NOTE You can uncomment 'hprintln' here and in the code below for a bit more
// verbosity at runtime, at the cost of throwing off the timing of the blink
// (using 'semihosting' for printing debug info anywhere slows program
// execution down)
//use cortex_m_semihosting::hprintln;

// A type definition for the GPIO pin to be used for our LED
// For the onboard nucleo LED, use gpioa::PA5 or gpiob::PB13 depending your model
type LedPin = gpiob::PB7<Output<PushPull>>;

// Make LED pin globally available
static G_LED: Mutex<RefCell<Option<LedPin>>> = Mutex::new(RefCell::new(None));

// Make timer interrupt registers globally available
static G_TIM: Mutex<RefCell<Option<CountDownTimer<TIM2>>>> = Mutex::new(RefCell::new(None));

// Define an interupt handler, i.e. function to call when interrupt occurs.
// This specific interrupt will "trip" when the timer TIM2 times out
#[interrupt]
fn TIM2() {
    static mut LED: Option<LedPin> = None;
    static mut TIM: Option<CountDownTimer<TIM2>> = None;

    let led = LED.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| {
            // Move LED pin here, leaving a None in its place
            G_LED.borrow(cs).replace(None).unwrap()
        })
    });

    let tim = TIM.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| {
            // Move LED pin here, leaving a None in its place
            G_TIM.borrow(cs).replace(None).unwrap()
        })
    });

    let _ = led.toggle();
    let _ = tim.wait();
}

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(16.mhz()).pclk1(8.mhz()).freeze();

    // Configure PA5 pin to blink LED
    let gpiob = dp.GPIOB.split();
    let mut led = gpiob.pb7.into_push_pull_output();
    let _ = led.set_high(); // Turn off

    // Move the pin into our global storage
    cortex_m::interrupt::free(|cs| *G_LED.borrow(cs).borrow_mut() = Some(led));

    // Set up a timer expiring after 1s
    let mut timer = Timer::new(dp.TIM2, &clocks).start_count_down(1.hz());

    // Generate an interrupt when the timer expires
    timer.listen(Event::TimeOut);

    // Move the timer into our global storage
    cortex_m::interrupt::free(|cs| *G_TIM.borrow(cs).borrow_mut() = Some(timer));

    //enable TIM2 interrupt
    unsafe {
        cortex_m::peripheral::NVIC::unmask(Interrupt::TIM2);
    }

    loop {
        wfi();
    }
}
