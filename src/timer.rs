//! Timers

use cast::{u16, u32};
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::SYST;
use embedded_hal::timer::{Cancel, CountDown, Periodic};
use void::Void;

#[cfg(any(feature = "stm32f205", feature = "stm32f215",))]
use crate::stm32::{
    TIM1, TIM10, TIM11, TIM12, TIM13, TIM14, TIM2, TIM3, TIM4, TIM5, TIM6, TIM7, TIM8, TIM9,
};
use crate::{bb, pac::RCC};

use crate::rcc::Clocks;
use crate::time::Hertz;

/// Hardware timers
pub struct Timer<TIM> {
    clocks: Clocks,
    tim: TIM,
}

/// Interrupt events
pub enum Event {
    /// Timer timed out / count down ended
    TimeOut,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    /// Timer is disabled
    Disabled,
}

impl Timer<SYST> {
    /// Configures the SYST clock as a periodic count down timer
    pub fn syst<T>(mut syst: SYST, timeout: T, clocks: Clocks) -> Self
    where
        T: Into<Hertz>,
    {
        syst.set_clock_source(SystClkSource::Core);
        let mut timer = Timer { tim: syst, clocks };
        timer.start(timeout);
        timer
    }

    /// Starts listening for an `event`
    pub fn listen(&mut self, event: Event) {
        match event {
            Event::TimeOut => self.tim.enable_interrupt(),
        }
    }

    /// Stops listening for an `event`
    pub fn unlisten(&mut self, event: Event) {
        match event {
            Event::TimeOut => self.tim.disable_interrupt(),
        }
    }
}

impl CountDown for Timer<SYST> {
    type Time = Hertz;

    fn start<T>(&mut self, timeout: T)
    where
        T: Into<Hertz>,
    {
        let rvr = self.clocks.sysclk().0 / timeout.into().0 - 1;

        assert!(rvr < (1 << 24));

        self.tim.set_reload(rvr);
        self.tim.clear_current();
        self.tim.enable_counter();
    }

    fn wait(&mut self) -> nb::Result<(), Void> {
        if self.tim.has_wrapped() {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl Cancel for Timer<SYST> {
    type Error = Error;

    fn cancel(&mut self) -> Result<(), Self::Error> {
        if !self.tim.is_counter_enabled() {
            return Err(Self::Error::Disabled);
        }

        self.tim.disable_counter();
        Ok(())
    }
}

impl Periodic for Timer<SYST> {}

macro_rules! hal {
    ($($TIM:ident: ($tim:ident, $en_bit:expr, $reset_bit:expr, $apbenr:ident, $apbrstr:ident, $pclk:ident, $ppre:ident),)+) => {
        $(
            impl Timer<$TIM> {
                /// Configures a TIM peripheral as a periodic count down timer
                pub fn $tim<T>(tim: $TIM, timeout: T, clocks: Clocks) -> Self
                where
                    T: Into<Hertz>,
                {
                    unsafe {
                        //NOTE(unsafe) this reference will only be used for atomic writes with no side effects
                        let rcc = &(*RCC::ptr());
                        // Enable and reset the timer peripheral, it's the same bit position for both registers
                        bb::set(&rcc.$apbenr, $en_bit);
                        bb::set(&rcc.$apbrstr, $reset_bit);
                        bb::clear(&rcc.$apbrstr, $reset_bit);
                    }

                    let mut timer = Timer {
                        clocks,
                        tim,
                    };
                    timer.start(timeout);

                    timer
                }

                /// Starts listening for an `event`
                ///
                /// Note, you will also have to enable the TIM2 interrupt in the NVIC to start
                /// receiving events.
                pub fn listen(&mut self, event: Event) {
                    match event {
                        Event::TimeOut => {
                            // Enable update event interrupt
                            self.tim.dier.write(|w| w.uie().set_bit());
                        }
                    }
                }

                /// Clears interrupt associated with `event`.
                ///
                /// If the interrupt is not cleared, it will immediately retrigger after
                /// the ISR has finished.
                pub fn clear_interrupt(&mut self, event: Event) {
                    match event {
                        Event::TimeOut => {
                            // Clear interrupt flag
                            self.tim.sr.write(|w| w.uif().clear_bit());
                        }
                    }
                }

                /// Stops listening for an `event`
                pub fn unlisten(&mut self, event: Event) {
                    match event {
                        Event::TimeOut => {
                            // Enable update event interrupt
                            self.tim.dier.write(|w| w.uie().clear_bit());
                        }
                    }
                }

                /// Releases the TIM peripheral
                pub fn release(self) -> $TIM {
                    // pause counter
                    self.tim.cr1.modify(|_, w| w.cen().clear_bit());
                    self.tim
                }
            }

            impl CountDown for Timer<$TIM> {
                type Time = Hertz;

                fn start<T>(&mut self, timeout: T)
                where
                    T: Into<Hertz>,
                {
                    // pause
                    self.tim.cr1.modify(|_, w| w.cen().clear_bit());
                    // reset counter
                    self.tim.cnt.reset();

                    let frequency = timeout.into().0;
                    let pclk_mul = if self.clocks.$ppre() == 1 { 1 } else { 2 };
                    let ticks = self.clocks.$pclk().0 * pclk_mul / frequency;

                    let psc = u16((ticks - 1) / (1 << 16)).unwrap();
                    self.tim.psc.write(|w| w.psc().bits(psc) );

                    let arr = u16(ticks / u32(psc + 1)).unwrap();
                    self.tim.arr.write(|w| unsafe { w.bits(u32(arr)) });

                    // start counter
                    self.tim.cr1.modify(|_, w| w.cen().set_bit());
                }

                fn wait(&mut self) -> nb::Result<(), Void> {
                    if self.tim.sr.read().uif().bit_is_clear() {
                        Err(nb::Error::WouldBlock)
                    } else {
                        self.tim.sr.modify(|_, w| w.uif().clear_bit());
                        Ok(())
                    }
                }
            }

            impl Cancel for Timer<$TIM>
            {
                type Error = Error;

                fn cancel(&mut self) -> Result<(), Self::Error> {
                    let is_counter_enabled = self.tim.cr1.read().cen().is_enabled();
                    if !is_counter_enabled {
                        return Err(Self::Error::Disabled);
                    }

                    // disable counter
                    self.tim.cr1.modify(|_, w| w.cen().clear_bit());
                    Ok(())
                }
            }

            impl Periodic for Timer<$TIM> {}
        )+
    }
}

hal! {
    TIM1: (tim1, 0, 0, apb2enr, apb2rstr, pclk2, ppre2),
    TIM2: (tim2, 0, 0, apb1enr, apb1rstr, pclk1, ppre1),
    TIM3: (tim3, 1, 1, apb1enr, apb1rstr, pclk1, ppre1),
    TIM4: (tim4, 2, 2, apb1enr, apb1rstr, pclk1, ppre1),
    TIM5: (tim5, 3, 3, apb1enr, apb1rstr, pclk1, ppre1),
    TIM6: (tim6, 4, 4, apb1enr, apb1rstr, pclk1, ppre1),
    TIM7: (tim7, 5, 5, apb1enr, apb1rstr, pclk1, ppre1),
    TIM8: (tim8, 1, 1, apb2enr, apb2rstr, pclk2, ppre2),
    TIM9: (tim9, 16, 16, apb2enr, apb2rstr, pclk2, ppre2),
    TIM10: (tim10, 17, 17, apb2enr, apb2rstr, pclk2, ppre2),
    TIM11: (tim11, 18, 18, apb2enr, apb2rstr, pclk2, ppre2),
    TIM12: (tim12, 6, 6, apb1enr, apb1rstr, pclk1, ppre1),
    TIM13: (tim13, 7, 7, apb1enr, apb1rstr, pclk1, ppre1),
    TIM14: (tim14, 8, 8, apb1enr, apb1rstr, pclk1, ppre1),
}

use crate::gpio::gpiob::*;
use crate::gpio::gpioc::*;
use crate::gpio::gpiod::*;
use crate::gpio::gpioe::*;
use crate::gpio::gpiof::*;

use crate::gpio::{gpioa::*, Alternate};
use crate::gpio::{AF1, AF2, AF3, AF9};

// Output channels marker traits
pub trait PinC1<TIM> {}
pub trait PinC2<TIM> {}
pub trait PinC3<TIM> {}
pub trait PinC4<TIM> {}

macro_rules! channel_impl {
    ( $( $TIM:ident, $PINC:ident, $PINX:ident, $MODE:ident<$AF:ident>; )+ ) => {
        $(
            impl $PINC<$TIM> for $PINX<$MODE<$AF>> {}
        )+
    };
}

channel_impl!(
  TIM1, PinC1, PA8, Alternate<AF1>;
  TIM1, PinC2, PA9, Alternate<AF1>;
  TIM1, PinC3, PA10, Alternate<AF1>;
  TIM1, PinC4, PA11, Alternate<AF1>;

  TIM1, PinC1, PE9, Alternate<AF1>;
  TIM1, PinC2, PE11, Alternate<AF1>;
  TIM1, PinC3, PE13, Alternate<AF1>;
  TIM1, PinC4, PE14, Alternate<AF1>;

  TIM2, PinC1, PA0, Alternate<AF1>;
  TIM2, PinC2, PA1, Alternate<AF1>;
  TIM2, PinC3, PA2, Alternate<AF1>;
  TIM2, PinC4, PA3, Alternate<AF1>;
  TIM2, PinC1, PA5, Alternate<AF1>;
  TIM2, PinC1, PA15, Alternate<AF1>;

  TIM2, PinC2, PB3, Alternate<AF1>;
  TIM2, PinC3, PB10, Alternate<AF1>;
  TIM2, PinC4, PB11, Alternate<AF1>;

  TIM3, PinC1, PA6, Alternate<AF2>;
  TIM3, PinC2, PA7, Alternate<AF2>;
  TIM3, PinC3, PB0, Alternate<AF2>;
  TIM3, PinC4, PB1, Alternate<AF2>;

  TIM3, PinC1, PB4, Alternate<AF2>;
  TIM3, PinC2, PB5, Alternate<AF2>;

  TIM3, PinC1, PC6, Alternate<AF2>;
  TIM3, PinC2, PC7, Alternate<AF2>;
  TIM3, PinC3, PC8, Alternate<AF2>;
  TIM3, PinC4, PC9, Alternate<AF2>;

  TIM4, PinC1, PB6, Alternate<AF2>;
  TIM4, PinC2, PB7, Alternate<AF2>;
  TIM4, PinC3, PB8, Alternate<AF2>;
  TIM4, PinC4, PB9, Alternate<AF2>;

  TIM4, PinC1, PD12, Alternate<AF2>;
  TIM4, PinC2, PD13, Alternate<AF2>;
  TIM4, PinC3, PD14, Alternate<AF2>;
  TIM4, PinC4, PD15, Alternate<AF2>;

  TIM5, PinC1, PA0, Alternate<AF2>;
  TIM5, PinC2, PA1, Alternate<AF2>;
  TIM5, PinC3, PA2, Alternate<AF2>;
  TIM5, PinC4, PA3, Alternate<AF2>;

  TIM5, PinC1, PF3, Alternate<AF2>;
  TIM5, PinC2, PF4, Alternate<AF2>;
  TIM5, PinC3, PF5, Alternate<AF2>;
  TIM5, PinC4, PF10, Alternate<AF2>;

  TIM8, PinC1, PC6, Alternate<AF3>;
  TIM8, PinC2, PC7, Alternate<AF3>;
  TIM8, PinC3, PC8, Alternate<AF3>;
  TIM8, PinC4, PC9, Alternate<AF3>;

  TIM9, PinC1, PA2, Alternate<AF3>;
  TIM9, PinC2, PA3, Alternate<AF3>;
  TIM9, PinC1, PE5, Alternate<AF3>;
  TIM9, PinC2, PE6, Alternate<AF3>;

  TIM10, PinC1, PB8, Alternate<AF3>;
  TIM10, PinC1, PF6, Alternate<AF3>;

  TIM11, PinC1, PB9, Alternate<AF3>;
  TIM11, PinC1, PF7, Alternate<AF3>;

  TIM12, PinC1, PB14, Alternate<AF9>;
  TIM12, PinC2, PB15, Alternate<AF9>;

  TIM13, PinC1, PA6, Alternate<AF9>;
  TIM13, PinC1, PA7, Alternate<AF9>;

  TIM14, PinC1, PF8, Alternate<AF9>;
  TIM14, PinC1, PF9, Alternate<AF9>;
);
