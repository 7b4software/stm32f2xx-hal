stm32f4xx-hal
=============

[![Crates.io](https://img.shields.io/crates/d/stm32f4xx-hal.svg)](https://crates.io/crates/stm32f2xx-hal)
[![Crates.io](https://img.shields.io/crates/v/stm32f4xx-hal.svg)](https://crates.io/crates/stm32f2xx-hal)
[![Released API docs](https://docs.rs/stm32f2xx-hal/badge.svg)](https://docs.rs/stm32f2xx-hal)
![Minimum Supported Rust Version](https://img.shields.io/badge/rustc-1.51+-blue.svg)

_stm32f2xx-hal_ contains a multi device hardware abstraction on top of the
peripheral access API for the STMicro STM32F2 series microcontrollers. The
selection of the MCU is done by feature gates, typically specified by board
support crates. Currently supported configurations are:

* stm32f205
* stm32f207
* stm32f215
* stm32f217

This crate is 99% based on stm32f4xx-hal with slightly modifications to make it work on stm32f2xx family of CPU's.

Collaboration on this crate is highly welcome as are pull requests!

[stm32f2]: https://crates.io/crates/stm32f2
[stm32f2xx-hal]: https://github.com/stm32-rs/stm32f4xx-hal
[embedded-hal]: https://github.com/rust-embedded/embedded-hal

Setting up your project
-------

Check if the BSP for your board exists in the
[stm32-rs](https://github.com/stm32-rs) page.
If it exists, the `stm32f4xx-hal` crate should be already included, so you can
use the bsp as BSP for your project.

Otherwise, create a new Rust project as you usually do with `cargo init`. The
"hello world" of embedded development is usually to blink a LED. The code to do
so is available in [examples/delay-syst-blinky.rs](examples/delay-syst-blinky.rs).
Copy that file to the `main.rs` of your project.

You also need to add some dependencies to your `Cargo.toml`:

```toml
[dependencies]
embedded-hal = "0.2"
nb = "1"
cortex-m = "0.7"
cortex-m-rt = "0.7"
# Panic behaviour, see https://crates.io/keywords/panic-impl for alternatives
panic-halt = "0.2"

[dependencies.stm32f4xx-hal]
version = "0.10"
features = ["rt", "stm32f205"] # replace the model of your microcontroller here
```

We also need to tell Rust how to link our executable and how to lay out the
result in memory. To accomplish all this, copy [.cargo/config](.cargo/config)
and [memory.x](memory.x) from the `stm32f4xx-hal` repository to your project and make sure the sizes match up with the datasheet. Also note that there might be different kinds of memory which are not equal; to be on the safe side only specify the size of the first block at the specified address.

License
-------

[0-clause BSD license](LICENSE-0BSD.txt).
