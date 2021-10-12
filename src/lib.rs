#![no_std]
#![allow(non_camel_case_types)]

#[cfg(not(feature = "device-selected"))]
compile_error!(
    "This crate requires one of the following device features enabled:
        stm32f205
        stm32f207
        stm32f215
        stm32f217"
);

#[cfg(feature = "device-selected")]
pub use embedded_hal as hal;

#[cfg(feature = "device-selected")]
pub use nb;
#[cfg(feature = "device-selected")]
pub use nb::block;

#[cfg(feature = "stm32f205")]
/// Re-export of the [svd2rust](https://crates.io/crates/svd2rust) auto-generated API for the stm32f405 peripherals.
pub use stm32f2::stm32f215 as pac;

#[cfg(feature = "stm32f207")]
/// Re-export of the [svd2rust](https://crates.io/crates/svd2rust) auto-generated API for the stm32f407 peripherals.
pub use stm32f2::stm32f207 as pac;

#[cfg(feature = "stm32f215")]
/// Re-export of the [svd2rust](https://crates.io/crates/svd2rust) auto-generated API for the stm32f405 peripherals.
pub use stm32f2::stm32f215 as pac;

#[cfg(feature = "stm32f217")]
/// Re-export of the [svd2rust](https://crates.io/crates/svd2rust) auto-generated API for the stm32f407 peripherals.
pub use stm32f2::stm32f217 as pac;

// Enable use of interrupt macro
#[cfg(feature = "rt")]
pub use crate::pac::interrupt;

#[cfg(feature = "device-selected")]
pub mod adc;
#[cfg(feature = "device-selected")]
pub mod bb;
#[cfg(all(
    feature = "device-selected",
    feature = "can",
    any(feature = "can1", feature = "can2",)
))]
pub mod can;
#[cfg(feature = "device-selected")]
pub mod crc32;
#[cfg(all(feature = "device-selected", feature = "dac"))]
pub mod dac;
#[cfg(feature = "device-selected")]
pub mod delay;
#[cfg(feature = "device-selected")]
pub mod gpio;
#[cfg(feature = "device-selected")]
pub mod i2c;
#[cfg(all(feature = "device-selected", feature = "i2s"))]
pub mod i2s;
#[cfg(all(feature = "device-selected", feature = "usb_fs", feature = "otg-fs"))]
pub mod otg_fs;
#[cfg(all(
    feature = "device-selected",
    any(feature = "usb_hs", docsrs),
    feature = "otg-hs",
))]
pub mod otg_hs;

#[cfg(all(feature = "device-selected", feature = "rng"))]
pub mod rng;

#[cfg(feature = "device-selected")]
pub mod dma;
#[cfg(feature = "device-selected")]
pub mod dwt;
#[cfg(feature = "device-selected")]
pub mod flash;
#[cfg(feature = "device-selected")]
pub mod prelude;
#[cfg(feature = "device-selected")]
pub mod pwm;
#[cfg(feature = "device-selected")]
pub mod pwm_input;
#[cfg(feature = "device-selected")]
pub mod qei;
#[cfg(feature = "device-selected")]
pub mod rcc;
//#[cfg(feature = "device-selected")]
//pub mod rtc;
#[cfg(all(feature = "device-selected", feature = "sdio-host", feature = "sdio"))]
pub mod sdio;
#[cfg(feature = "device-selected")]
pub mod serial;
#[cfg(feature = "device-selected")]
pub mod signature;
#[cfg(feature = "device-selected")]
pub mod spi;
#[cfg(feature = "device-selected")]
pub mod syscfg;
#[cfg(feature = "device-selected")]
pub mod time;
#[cfg(feature = "device-selected")]
pub mod timer;
#[cfg(feature = "device-selected")]
pub mod watchdog;

#[cfg(feature = "device-selected")]
mod sealed {
    pub trait Sealed {}
}
#[cfg(feature = "device-selected")]
pub(crate) use sealed::Sealed;
