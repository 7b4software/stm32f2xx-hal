//! Demonstrate the use of a blocking `Delay` using the SYST (sysclock) timer.

#![deny(unsafe_code)]
#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

// Halt on panic
use panic_halt as _; // panic handler

use cortex_m_rt::entry;
use stm32f2xx_hal as hal;

use crate::hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        // Set up the blue LED. On the Nucleo-207RE it's connected to pin PB7.
        let gpiob = dp.GPIOB.split();
        let mut blue = gpiob.pb7.into_push_pull_output();
        // Set up the blue LED. On the Nucleo-207RE it's connected to pin PB14.
        let mut red = gpiob.pb14.into_push_pull_output();

        blue.set_high();
        red.set_high();
        // Set up the system clock. We want to run at 48MHz for this one.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(16.mhz()).freeze();

        // Create a delay abstraction based on SysTick
        let mut delay = hal::delay::Delay::new(cp.SYST, &clocks);

        loop {
            // On for 1s, off for 1s.
            red.set_high();
            delay.delay_ms(1000_u32);
            red.set_low();
            delay.delay_ms(1000_u32);
        }
    }

    loop {}
}
