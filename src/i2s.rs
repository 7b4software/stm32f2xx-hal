//! I2S (inter-IC Sound) communication using SPI peripherals
//!
//! This module is only available if the `i2s` feature is enabled.

use crate::gpio::{Const, NoPin, SetAlternate};
use stm32_i2s_v12x::{Instance, RegisterBlock};

use crate::pac::RCC;
use crate::rcc;
use crate::time::Hertz;
use crate::{rcc::Clocks, spi};

// I2S pins are mostly the same as the corresponding SPI pins:
// MOSI -> SD
// NSS -> WS (the current SPI code doesn't define NSS pins)
// SCK -> CK
// The master clock output is separate.

/// A pin that can be used as SD (serial data)
pub trait PinSd<SPI> {
    type A;
}
/// A pin that can be used as WS (word select, left/right clock)
pub trait PinWs<SPI> {
    type A;
}
/// A pin that can be used as CK (bit clock)
pub trait PinCk<SPI> {
    type A;
}
/// A pin that can be used as MCK (master clock output)
pub trait PinMck<SPI> {
    type A;
}

impl<SPI> PinMck<SPI> for NoPin
where
    SPI: Instance,
{
    type A = Const<0>;
}

/// Each MOSI pin can also be used as SD
impl<P, SPI, const MOSIA: u8> PinSd<SPI> for P
where
    P: spi::PinMosi<SPI, A = Const<MOSIA>>,
{
    type A = Const<MOSIA>;
}
/// Each SCK pin can also be used as CK
impl<P, SPI, const SCKA: u8> PinCk<SPI> for P
where
    P: spi::PinSck<SPI, A = Const<SCKA>>,
{
    type A = Const<SCKA>;
}

/// A placeholder for when the MCLK pin is not needed
pub type NoMasterClock = NoPin;

/// A set of pins configured for I2S communication: (WS, CK, MCLK, SD)
///
/// NoMasterClock can be used instead of the master clock pin.
pub trait Pins<SPI> {}

impl<SPI, PWS, PCK, PMCLK, PSD> Pins<SPI> for (PWS, PCK, PMCLK, PSD)
where
    PWS: PinWs<SPI>,
    PCK: PinCk<SPI>,
    PMCLK: PinMck<SPI>,
    PSD: PinSd<SPI>,
{
}

/// Master clock (MCK) pins
mod mck_pins {
    macro_rules! pin_mck {
        ($($PER:ident => $pin:ident<$af:literal>,)+) => {
            $(
                impl<MODE> crate::i2s::PinMck<$PER> for $pin<MODE> {
                    type A = crate::gpio::Const<$af>;
                }
            )+
        };
    }

    mod common {
        use crate::gpio::gpioc::PC6;
        use crate::pac::SPI2;
        // All STM32F2 models support PC6<5> for SPI2/I2S2
        pin_mck! { SPI2 => PC6<5>, }
    }

    // On all models except the STM32F410, PC7<6> is the master clock output from I2S3.
    #[cfg(feature = "spi3")]
    mod i2s3_pc7_af6 {
        use crate::gpio::gpioc::PC7;
        use crate::pac::SPI3;
        pin_mck! { SPI3 => PC7<6>, }
    }
}

/// Word select (WS) pins
mod ws_pins {
    macro_rules! pin_ws {
        ($($PER:ident => $pin:ident<$af:literal>,)+) => {
            $(
                impl<MODE> crate::i2s::PinWs<$PER> for $pin<MODE> {
                    type A = crate::gpio::Const<$af>;
                }
            )+
        };
    }

    mod common {
        use crate::gpio::gpioa::{PA15, PA4};
        use crate::gpio::gpiob::{PB12, PB9};
        use crate::gpio::gpioi::PI0;
        use crate::pac::{SPI2, SPI3};
        // All STM32F2 models support these pins
        pin_ws! {
           SPI2 => PB9<5>,
           SPI2 => PB12<5>,
           SPI3 => PA4<6>,
           SPI3 => PA15<6>,
           SPI2 => PI0<5>,
        }
        use crate::gpio::gpioi::PI0;
        use crate::pac::SPI2;
    }
}

pub trait I2sFreq {
    fn i2s_freq(clocks: &Clocks) -> Hertz;
}

/// Implements Instance for I2s<$SPIX, _> and creates an I2s::$spix function to create and enable
/// the peripheral
///
/// $SPIX: The fully-capitalized name of the SPI peripheral (example: SPI1)
/// $i2sx: The lowercase I2S name of the peripheral (example: i2s1). This is the name of the
/// function that creates an I2s and enables the peripheral clock.
/// $clock: The name of the Clocks function that returns the frequency of the I2S clock input
/// to this SPI peripheral (i2s_cl, i2s_apb1_clk, or i2s2_apb_clk)
macro_rules! i2s {
    ($SPIX:ty, $clock:ident) => {
        impl I2sFreq for $SPIX {
            fn i2s_freq(clocks: &Clocks) -> Hertz {
                clocks
                    .$clock()
                    .expect("I2S clock input for SPI not enabled")
            }
        }

        unsafe impl<PINS> Instance for I2s<$SPIX, PINS> {
            const REGISTERS: *mut RegisterBlock = <$SPIX>::ptr() as *mut _;
        }
    };
}

impl<SPI, WS, CK, MCLK, SD, const WSA: u8, const CKA: u8, const MCLKA: u8, const SDA: u8>
    I2s<SPI, (WS, CK, MCLK, SD)>
where
    SPI: I2sFreq + rcc::Enable + rcc::Reset,
    WS: PinWs<SPI, A = Const<WSA>> + SetAlternate<WSA>,
    CK: PinCk<SPI, A = Const<CKA>> + SetAlternate<CKA>,
    MCLK: PinMck<SPI, A = Const<MCLKA>> + SetAlternate<MCLKA>,
    SD: PinSd<SPI, A = Const<SDA>> + SetAlternate<SDA>,
{
    /// Creates an I2s object around an SPI peripheral and pins
    ///
    /// This function enables and resets the SPI peripheral, but does not configure it.
    ///
    /// The returned I2s object implements [stm32_i2s_v12x::Instance], so it can be used
    /// to configure the peripheral and communicate.
    ///
    /// # Panics
    ///
    /// This function panics if the I2S clock input (from the I2S PLL or similar)
    /// is not configured.
    pub fn new(spi: SPI, mut pins: (WS, CK, MCLK, SD), clocks: Clocks) -> Self {
        let input_clock = SPI::i2s_freq(&clocks);
        unsafe {
            // NOTE(unsafe) this reference will only be used for atomic writes with no side effects.
            let rcc = &(*RCC::ptr());
            // Enable clock, enable reset, clear, reset
            SPI::enable(rcc);
            SPI::reset(rcc);
        }

        pins.0.set_alt_mode();
        pins.1.set_alt_mode();
        pins.2.set_alt_mode();
        pins.3.set_alt_mode();

        I2s {
            _spi: spi,
            _pins: pins,
            input_clock,
        }
    }
}

i2s!(crate::pac::SPI3, i2s_clk);
/// An I2s wrapper around an SPI object and pins
pub struct I2s<I, PINS> {
    _spi: I,
    _pins: PINS,
    /// Frequency of clock input to this peripheral from the I2S PLL or related source
    input_clock: Hertz,
}

impl<I, PINS> I2s<I, PINS> {
    /// Returns the frequency of the clock signal that the SPI peripheral is receiving from the
    /// I2S PLL or similar source
    pub fn input_clock(&self) -> Hertz {
        self.input_clock
    }
}

// DMA support: reuse existing mappings for SPI
mod dma {
    use super::*;
    use crate::dma::traits::{DMASet, PeriAddress};
    use core::ops::Deref;

    /// I2S DMA reads from and writes to the data register
    unsafe impl<SPI, PINS, MODE> PeriAddress for stm32_i2s_v12x::I2s<I2s<SPI, PINS>, MODE>
    where
        I2s<SPI, PINS>: Instance,
        PINS: Pins<SPI>,
        SPI: Deref<Target = crate::pac::spi1::RegisterBlock>,
    {
        /// SPI_DR is only 16 bits. Multiple transfers are needed for a 24-bit or 32-bit sample,
        /// as explained in the reference manual.
        type MemSize = u16;

        fn address(&self) -> u32 {
            let registers = &*self.instance()._spi;
            &registers.dr as *const _ as u32
        }
    }

    /// DMA is available for I2S based on the underlying implementations for SPI
    unsafe impl<SPI, PINS, MODE, STREAM, DIR, const CHANNEL: u8> DMASet<STREAM, DIR, CHANNEL>
        for stm32_i2s_v12x::I2s<I2s<SPI, PINS>, MODE>
    where
        SPI: DMASet<STREAM, DIR, CHANNEL>,
    {
    }
}
