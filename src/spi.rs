use core::ops::Deref;
use core::ptr;

use crate::bb;
use embedded_hal::spi;
pub use embedded_hal::spi::{Mode, Phase, Polarity};

#[cfg(any(feature = "stm32f205", feature = "stm32f215"))]
use crate::stm32::{spi1, RCC, SPI1, SPI2, SPI3};

#[cfg(any(feature = "stm32f205", feature = "stm32f215"))]
use crate::gpio::gpioa::{PA5, PA6, PA7};

#[cfg(any(feature = "stm32f205", feature = "stm32f215"))]
use crate::gpio::gpiob::{PB10, PB13, PB14, PB15, PB3, PB4, PB5};

#[cfg(any(feature = "stm32f205", feature = "stm32f215"))]
use crate::gpio::gpioc::{PC10, PC11, PC12, PC2, PC3};

#[cfg(any(feature = "stm32f205", feature = "stm32f215"))]
use crate::gpio::gpioi::{PI1, PI2, PI3};

#[cfg(any(feature = "stm32f205", feature = "stm32f215"))]
use crate::gpio::{Alternate, AF5, AF6};

use crate::rcc::Clocks;
use crate::time::Hertz;

/// SPI error
#[derive(Debug)]
pub enum Error {
    /// Overrun occurred
    Overrun,
    /// Mode fault occurred
    ModeFault,
    /// CRC error
    Crc,
    #[doc(hidden)]
    _Extensible,
}

pub trait Pins<SPI> {}
pub trait PinSck<SPI> {}
pub trait PinMiso<SPI> {}
pub trait PinMosi<SPI> {}

impl<SPI, SCK, MISO, MOSI> Pins<SPI> for (SCK, MISO, MOSI)
where
    SCK: PinSck<SPI>,
    MISO: PinMiso<SPI>,
    MOSI: PinMosi<SPI>,
{
}

/// A filler type for when the SCK pin is unnecessary
pub struct NoSck;
/// A filler type for when the Miso pin is unnecessary
pub struct NoMiso;
/// A filler type for when the Mosi pin is unnecessary
pub struct NoMosi;

macro_rules! pins {
    ($($SPIX:ty: SCK: [$($SCK:ty),*] MISO: [$($MISO:ty),*] MOSI: [$($MOSI:ty),*])+) => {
        $(
            $(
                impl PinSck<$SPIX> for $SCK {}
            )*
            $(
                impl PinMiso<$SPIX> for $MISO {}
            )*
            $(
                impl PinMosi<$SPIX> for $MOSI {}
            )*
        )+
    }
}

#[cfg(any(feature = "stm32f205", feature = "stm32f207"))]
pins! {
    SPI1:
        SCK: [
            NoSck,
            PA5<Alternate<AF5>>,
            PB3<Alternate<AF5>>
        ]
        MISO: [
            NoMiso,
            PA6<Alternate<AF5>>,
            PB4<Alternate<AF5>>
        ]
        MOSI: [
            NoMosi,
            PA7<Alternate<AF5>>,
            PB5<Alternate<AF5>>
        ]

    SPI2:
        SCK: [
            NoSck,
            PB10<Alternate<AF5>>,
            PB13<Alternate<AF5>>
        ]
        MISO: [
            NoMiso,
            PB14<Alternate<AF5>>,
            PC2<Alternate<AF5>>
        ]
        MOSI: [
            NoMosi,
            PB15<Alternate<AF5>>,
            PC3<Alternate<AF5>>
        ]
    SPI3:
        SCK: [
            NoSck,
            PB3<Alternate<AF6>>,
            PC10<Alternate<AF6>>
        ]
        MISO: [
            NoMiso,
            PB4<Alternate<AF6>>,
            PC11<Alternate<AF6>>
        ]
        MOSI: [
            NoMosi,
            PB5<Alternate<AF6>>,
            PC12<Alternate<AF6>>
        ]
}

#[cfg(any(feature = "stm32f205",))]
pins! {
    SPI2:
        SCK: [PI1<Alternate<AF5>>]
        MISO: [PI2<Alternate<AF5>>]
        MOSI: [PI3<Alternate<AF5>>]
}

/// Interrupt events
pub enum Event {
    /// New data has been received
    Rxne,
    /// Data can be sent
    Txe,
    /// An error occurred
    Error,
}

#[derive(Debug)]
pub struct Spi<SPI, PINS> {
    spi: SPI,
    pins: PINS,
}

#[cfg(any(feature = "stm32f205",))]
impl<PINS> Spi<SPI1, PINS> {
    pub fn spi1(spi: SPI1, pins: PINS, mode: Mode, freq: Hertz, clocks: Clocks) -> Self
    where
        PINS: Pins<SPI1>,
    {
        unsafe {
            const EN_BIT: u8 = 12;
            // NOTE(unsafe) this reference will only be used for atomic writes with no side effects.
            let rcc = &(*RCC::ptr());

            // Enable clock.
            bb::set(&rcc.apb2enr, EN_BIT);
        }

        Spi { spi, pins }.init(mode, freq, clocks.pclk2())
    }
}

#[cfg(any(feature = "stm32f205",))]
impl<PINS> Spi<SPI2, PINS> {
    pub fn spi2(spi: SPI2, pins: PINS, mode: Mode, freq: Hertz, clocks: Clocks) -> Self
    where
        PINS: Pins<SPI2>,
    {
        unsafe {
            const EN_BIT: u8 = 14;
            // NOTE(unsafe) this reference will only be used for atomic writes with no side effects.
            let rcc = &(*RCC::ptr());

            // Enable clock.
            bb::set(&rcc.apb1enr, EN_BIT);
        }

        Spi { spi, pins }.init(mode, freq, clocks.pclk1())
    }
}

#[cfg(any(feature = "stm32f205",))]
impl<PINS> Spi<SPI3, PINS> {
    pub fn spi3(spi: SPI3, pins: PINS, mode: Mode, freq: Hertz, clocks: Clocks) -> Self
    where
        PINS: Pins<SPI3>,
    {
        unsafe {
            const EN_BIT: u8 = 15;
            // NOTE(unsafe) this reference will only be used for atomic writes with no side effects.
            let rcc = &(*RCC::ptr());

            // Enable clock.
            bb::set(&rcc.apb1enr, EN_BIT);
        }

        Spi { spi, pins }.init(mode, freq, clocks.pclk1())
    }
}

impl<SPI, PINS> Spi<SPI, PINS>
where
    SPI: Deref<Target = spi1::RegisterBlock>,
{
    pub fn init(self, mode: Mode, freq: Hertz, clock: Hertz) -> Self {
        // disable SS output
        self.spi.cr2.write(|w| w.ssoe().clear_bit());

        let br = match clock.0 / freq.0 {
            0 => unreachable!(),
            1..=2 => 0b000,
            3..=5 => 0b001,
            6..=11 => 0b010,
            12..=23 => 0b011,
            24..=47 => 0b100,
            48..=95 => 0b101,
            96..=191 => 0b110,
            _ => 0b111,
        };

        // mstr: master configuration
        // lsbfirst: MSB first
        // ssm: enable software slave management (NSS pin free for other uses)
        // ssi: set nss high = master mode
        // dff: 8 bit frames
        // bidimode: 2-line unidirectional
        // spe: enable the SPI bus
        self.spi.cr1.write(|w| {
            w.cpha()
                .bit(mode.phase == Phase::CaptureOnSecondTransition)
                .cpol()
                .bit(mode.polarity == Polarity::IdleHigh)
                .mstr()
                .set_bit()
                .br()
                .bits(br)
                .lsbfirst()
                .clear_bit()
                .ssm()
                .set_bit()
                .ssi()
                .set_bit()
                .rxonly()
                .clear_bit()
                .dff()
                .clear_bit()
                .bidimode()
                .clear_bit()
                .spe()
                .set_bit()
        });

        self
    }

    /// Enable interrupts for the given `event`:
    ///  - Received data ready to be read (RXNE)
    ///  - Transmit data register empty (TXE)
    ///  - Transfer error
    pub fn listen(&mut self, event: Event) {
        match event {
            Event::Rxne => self.spi.cr2.modify(|_, w| w.rxneie().set_bit()),
            Event::Txe => self.spi.cr2.modify(|_, w| w.txeie().set_bit()),
            Event::Error => self.spi.cr2.modify(|_, w| w.errie().set_bit()),
        }
    }

    /// Disable interrupts for the given `event`:
    ///  - Received data ready to be read (RXNE)
    ///  - Transmit data register empty (TXE)
    ///  - Transfer error
    pub fn unlisten(&mut self, event: Event) {
        match event {
            Event::Rxne => self.spi.cr2.modify(|_, w| w.rxneie().clear_bit()),
            Event::Txe => self.spi.cr2.modify(|_, w| w.txeie().clear_bit()),
            Event::Error => self.spi.cr2.modify(|_, w| w.errie().clear_bit()),
        }
    }

    /// Return `true` if the TXE flag is set, i.e. new data to transmit
    /// can be written to the SPI.
    pub fn is_txe(&self) -> bool {
        self.spi.sr.read().txe().bit_is_set()
    }

    /// Return `true` if the RXNE flag is set, i.e. new data has been received
    /// and can be read from the SPI.
    pub fn is_rxne(&self) -> bool {
        self.spi.sr.read().rxne().bit_is_set()
    }

    /// Return `true` if the MODF flag is set, i.e. the SPI has experienced a
    /// Master Mode Fault. (see chapter 28.3.10 of the STM32F4 Reference Manual)
    pub fn is_modf(&self) -> bool {
        self.spi.sr.read().modf().bit_is_set()
    }

    /// Return `true` if the OVR flag is set, i.e. new data has been received
    /// while the receive data register was already filled.
    pub fn is_ovr(&self) -> bool {
        self.spi.sr.read().ovr().bit_is_set()
    }

    pub fn free(self) -> (SPI, PINS) {
        (self.spi, self.pins)
    }
}

impl<SPI, PINS> spi::FullDuplex<u8> for Spi<SPI, PINS>
where
    SPI: Deref<Target = spi1::RegisterBlock>,
{
    type Error = Error;

    fn read(&mut self) -> nb::Result<u8, Error> {
        let sr = self.spi.sr.read();

        Err(if sr.ovr().bit_is_set() {
            nb::Error::Other(Error::Overrun)
        } else if sr.modf().bit_is_set() {
            nb::Error::Other(Error::ModeFault)
        } else if sr.crcerr().bit_is_set() {
            nb::Error::Other(Error::Crc)
        } else if sr.rxne().bit_is_set() {
            // NOTE(read_volatile) read only 1 byte (the svd2rust API only allows
            // reading a half-word)
            return Ok(unsafe { ptr::read_volatile(&self.spi.dr as *const _ as *const u8) });
        } else {
            nb::Error::WouldBlock
        })
    }

    fn send(&mut self, byte: u8) -> nb::Result<(), Error> {
        let sr = self.spi.sr.read();

        Err(if sr.ovr().bit_is_set() {
            nb::Error::Other(Error::Overrun)
        } else if sr.modf().bit_is_set() {
            nb::Error::Other(Error::ModeFault)
        } else if sr.crcerr().bit_is_set() {
            nb::Error::Other(Error::Crc)
        } else if sr.txe().bit_is_set() {
            // NOTE(write_volatile) see note above
            unsafe { ptr::write_volatile(&self.spi.dr as *const _ as *mut u8, byte) }
            return Ok(());
        } else {
            nb::Error::WouldBlock
        })
    }
}

impl<SPI, PINS> embedded_hal::blocking::spi::transfer::Default<u8> for Spi<SPI, PINS> where
    SPI: Deref<Target = spi1::RegisterBlock>
{
}

impl<SPI, PINS> embedded_hal::blocking::spi::write::Default<u8> for Spi<SPI, PINS> where
    SPI: Deref<Target = spi1::RegisterBlock>
{
}
