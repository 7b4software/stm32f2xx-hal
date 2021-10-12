use super::*;
use crate::bb;

macro_rules! bus_enable {
    ($PER:ident => ($busX:ty, $bit:literal)) => {
        impl Enable for crate::pac::$PER {
            #[inline(always)]
            fn enable(rcc: &RccRB) {
                unsafe {
                    bb::set(Self::Bus::enr(rcc), $bit);
                }
                // Stall the pipeline to work around erratum 2.1.13 (DM00037591)
                cortex_m::asm::dsb();
            }
            #[inline(always)]
            fn disable(rcc: &RccRB) {
                unsafe {
                    bb::clear(Self::Bus::enr(rcc), $bit);
                }
            }
        }
    };
}
macro_rules! bus_lpenable {
    ($PER:ident => ($busX:ty, $bit:literal)) => {
        impl LPEnable for crate::pac::$PER {
            #[inline(always)]
            fn low_power_enable(rcc: &RccRB) {
                unsafe {
                    bb::set(Self::Bus::lpenr(rcc), $bit);
                }
                // Stall the pipeline to work around erratum 2.1.13 (DM00037591)
                cortex_m::asm::dsb();
            }
            #[inline(always)]
            fn low_power_disable(rcc: &RccRB) {
                unsafe {
                    bb::clear(Self::Bus::lpenr(rcc), $bit);
                }
            }
        }
    };
}
macro_rules! bus_reset {
    ($PER:ident => ($busX:ty, $bit:literal)) => {
        impl Reset for crate::pac::$PER {
            #[inline(always)]
            fn reset(rcc: &RccRB) {
                unsafe {
                    bb::set(Self::Bus::rstr(rcc), $bit);
                    bb::clear(Self::Bus::rstr(rcc), $bit);
                }
            }
        }
    };
}

macro_rules! bus {
    ($($PER:ident => ($busX:ty, $bit:literal),)+) => {
        $(
            impl crate::Sealed for crate::pac::$PER {}
            impl RccBus for crate::pac::$PER {
                type Bus = $busX;
            }
            bus_enable!($PER => ($busX, $bit));
            bus_lpenable!($PER => ($busX, $bit));
            bus_reset!($PER => ($busX, $bit));
        )+
    }
}

bus! {
    CRC => (AHB1, 12),
    DMA1 => (AHB1, 21),
    DMA2 => (AHB1, 22),
}

bus! {
    GPIOA => (AHB1, 0),
    GPIOB => (AHB1, 1),
    GPIOC => (AHB1, 2),
    GPIOH => (AHB1, 7),
}

bus! {
    GPIOD => (AHB1, 3),
    GPIOE => (AHB1, 4),
}
bus! {
    GPIOF => (AHB1, 5),
    GPIOG => (AHB1, 6),
}

bus! {
    GPIOI => (AHB1, 8),
}

bus! {
    RNG => (AHB2, 6),
}

#[cfg(feature = "otg-fs")]
bus! {
    OTG_FS_GLOBAL => (AHB2, 7),
}

#[cfg(feature = "otg-hs")]
bus! {
    OTG_HS_GLOBAL => (AHB1, 29),
}

#[cfg(feature = "fmc")]
bus! {
    FMC => (AHB3, 0),
}

bus! {
    PWR => (APB1, 28),
}

bus! {
    SPI1 => (APB2, 12),
    SPI2 => (APB1, 14),
}
#[cfg(feature = "spi3")]
bus! {
    SPI3 => (APB1, 15),
}

bus! {
    I2C1 => (APB1, 21),
    I2C2 => (APB1, 22),
}
#[cfg(feature = "i2c3")]
bus! {
    I2C3 => (APB1, 23),
}
#[cfg(feature = "fmpi2c1")]
bus! {
    FMPI2C1 => (APB1, 24),
}

bus! {
    USART1 => (APB2, 4),
    USART2 => (APB1, 17),
    USART3 => (APB1, 18),
    USART6 => (APB2, 5),
}
bus! {
    UART4 => (APB1, 19),
    UART5 => (APB1, 20),
}

#[cfg(any(feature = "can1", feature = "can2"))]
bus! {
    CAN1 => (APB1, 25),
    CAN2 => (APB1, 26),
}
#[cfg(feature = "dac")]
bus! {
    DAC => (APB1, 29),
}

bus! {
    SYSCFG => (APB2, 14),
}

bus! {
    ADC1 => (APB2, 8),
}

impl crate::Sealed for crate::pac::ADC2 {}
impl RccBus for crate::pac::ADC2 {
    type Bus = APB2;
}
bus_enable!(ADC2 => (APB2, 9));
bus_lpenable!(ADC2 => (APB2, 9));
bus_reset!(ADC2 => (APB2, 8));

impl crate::Sealed for crate::pac::ADC3 {}
impl RccBus for crate::pac::ADC3 {
    type Bus = APB2;
}
bus_enable!(ADC3 => (APB2, 10));
bus_lpenable!(ADC3 => (APB2, 10));
bus_reset!(ADC3 => (APB2, 8));

#[cfg(feature = "sdio")]
bus! {
    SDIO => (APB2, 11),
}

bus! {
    TIM1 => (APB2, 0),
    TIM2 => (APB1, 0),
    TIM3 => (APB1, 1),
    TIM4 => (APB1, 2),
    TIM5 => (APB1, 3),
    TIM6 => (APB1, 4),
    TIM7 => (APB1, 5),
    TIM8 => (APB2, 1),
    TIM9 => (APB2, 16),
    TIM10 => (APB2, 17),
    TIM11 => (APB2, 18),
    TIM12 => (APB1, 6),
    TIM13 => (APB1, 7),
    TIM14 => (APB1, 8),
}
