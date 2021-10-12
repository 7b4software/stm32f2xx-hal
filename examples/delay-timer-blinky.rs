//! Demonstrate the use of a blocking `Delay` using TIM5 general-purpose timer.
//! On nucleo stmf207

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
    if let (Some(dp), Some(_cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        let gpiob = dp.GPIOB.split();
        let mut blue = gpiob.pb7.into_push_pull_output();

        // Set up the system clock. We want to run at 16MHz for this one.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(16.mhz()).freeze();

        // Create a delay abstraction based on general-pupose 32-bit timer TIM5
        let mut delay = hal::delay::Delay::tim5(dp.TIM5, &clocks);

        loop {
            // On for 1s, off for 1s.
            blue.set_high();
            delay.delay_ms(1_000_u32);
            led.set_low();
            blue.delay_us(1_000_000_u32);
        }
    }

    loop {}
}
