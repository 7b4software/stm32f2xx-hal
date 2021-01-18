#![no_std]
#![allow(non_camel_case_types)]

#[cfg(not(feature = "device-selected"))]
compile_error!(
    "This crate requires one of the following device features enabled:
        stm32f205
        stm32f215
        stm32f207
        stm32f217"
);

pub use embedded_hal as hal;

pub use nb;
pub use nb::block;

#[cfg(any(feature = "stm32f205", feature = "stm32f215"))]
pub use stm32f2::stm32f215 as stm32;

#[cfg(any(feature = "stm32f207", feature = "stm32f217"))]
pub use stm32f2::stm32f217 as stm32;

// Enable use of interrupt macro
#[cfg(feature = "rt")]
pub use crate::stm32::interrupt;

//#[cfg(feature = "device-selected")]
//pub mod adc;
#[cfg(feature = "device-selected")]
pub mod bb;
#[cfg(feature = "device-selected")]
pub mod delay;
#[cfg(feature = "device-selected")]
pub mod gpio;
#[cfg(feature = "device-selected")]
pub mod i2c;
#[cfg(all(feature = "usb_fs", any(
    feature = "stm32f205",
    feature = "stm32f215",
    feature = "stm32f207",
    feature = "stm32f217",
)))]
pub mod otg_fs;
#[cfg(all(
    any(feature = "usb_hs", docsrs),
    any(
        feature = "stm32f205",
        feature = "stm32f215",
        feature = "stm32f207",
        feature = "stm32f217",
    )
))]
pub mod otg_hs;

#[cfg(feature = "device-selected")]
pub use stm32 as pac;

//#[cfg(feature = "device-selected")]
//pub mod dma;
//#[cfg(feature = "device-selected")]
//pub mod dwt;
#[cfg(feature = "device-selected")]
pub mod prelude;
#[cfg(feature = "device-selected")]
pub mod pwm;
#[cfg(feature = "device-selected")]
pub mod rcc;
#[cfg(feature = "device-selected")]
//pub mod serial;
//#[cfg(feature = "device-selected")]
pub mod signature;
//#[cfg(feature = "device-selected")]
//pub mod spi;
#[cfg(feature = "device-selected")]
pub mod time;
#[cfg(feature = "device-selected")]
pub mod timer;
#[cfg(feature = "device-selected")]
pub mod watchdog;
