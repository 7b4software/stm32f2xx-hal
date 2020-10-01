stm32f2xx-hal
=============

[![Crates.io](https://img.shields.io/crates/v/stm32f2xx-hal.svg)](https://crates.io/crates/stm32f2xx-hal)
[![Released API docs](https://docs.rs/stm32f4xx-hal/badge.svg)](https://docs.rs/stm32f2xx-hal)

*Experimental not ready for production HAL crate for STM32F2 heavily based on stm32f4xx-hal*

*Note the RCC is experimental and does not work with default implmentation also examples are not working since their are not fixed for stm32f2*

# TODO before release on crates.io

 - Fix RCC
 - Fix examples

_stm32f2xx-hal_ contains a multi device hardware abstraction on top of the
peripheral access API for the STMicro STM32F2 series microcontrollers. The
selection of the MCU is done by feature gates, typically specified by board
support crates. Currently supported configurations are:

* stm32f205
* stm32f215

The idea behind this crate is to gloss over the slight differences in the
various peripherals available on those MCUs so a HAL can be written for all
chips in that same family without having to cut and paste crates for every
single model.

Collaboration on this crate is highly welcome as are pull requests!

This crate relies on Adam Greigs fantastic [stm32f4][] crate to provide
appropriate register definitions and implements a partial set of the
[embedded-hal][] traits.

Most of the of the implementation was shamelessly adapted from the [stm32f40xx-hal][]
crate by Jorge Aparicio.

[stm32f2]: https://crates.io/crates/stm32f2
[stm32f4xx-hal]: https://crates.io/crates/stm32f4xx-hal
[embedded-hal]: https://github.com/rust-embedded/embedded-hal

Setting up your project
-------

Check if the BSP for your board exists in the
[stm32-rs](https://github.com/stm32-rs) page.
If it exists, the `stm32f2xx-hal` crate should be already included, so you can
use the bsp as BSP for your project.

Otherwise, create a new Rust project as you usually do with `cargo init`. The
"hello world" of embedded development is usually to blink a LED. The code to do
so is available in [examples/delay-blinky.rs](examples/delay-blinky.rs).
Copy that file to the `main.rs` of your project.

You also need to add some dependencies to your `Cargo.toml`:

```toml
[dependencies]
embedded-hal = "0.2"
nb = "0.1.2"
cortex-m = "0.6"
cortex-m-rt = "0.6"
# Panic behaviour, see https://crates.io/keywords/panic-impl for alternatives
panic-halt = "0.2"

[dependencies.stm32f2xx-hal]
version = "0.8"
features = ["rt", "stm32f407"] # replace the model of your microcontroller here
```

We also need to tell Rust how to link our executable and how to lay out the
result in memory. To accomplish all this, copy [.cargo/config](.cargo/config)
and [memory.x](memory.x) from the `stm32f4xx-hal` repository to your project and make sure the sizes match up with the datasheet. Also note that there might be different kinds of memory which are not equal; to be on the safe side only specify the size of the first block at the specified address.

License
-------

[0-clause BSD license](LICENSE-0BSD.txt).
