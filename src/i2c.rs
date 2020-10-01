use core::ops::Deref;
use embedded_hal::blocking::i2c::{Read, Write, WriteRead};

use crate::gpio::{AlternateOD, AF4};
#[cfg(any(feature = "stm32f205", feature = "stm32f215",))]
use crate::stm32::{I2C1, I2C2, I2C3, RCC};
use crate::{bb, pac::i2c1};

#[cfg(any(feature = "stm32f205", feature = "stm32f215",))]
use crate::gpio::gpioa::{PA8, PA9};

#[cfg(any(feature = "stm32f205", feature = "stm32f215",))]
use crate::gpio::gpiob::{PB10, PB11, PB6, PB7};

use crate::rcc::Clocks;
use crate::time::{Hertz, KiloHertz, U32Ext};

/// I2C abstraction
pub struct I2c<I2C, PINS> {
    i2c: I2C,
    pins: PINS,
}

pub trait Pins<I2c> {}
pub trait PinScl<I2c> {}
pub trait PinSda<I2c> {}

impl<I2c, SCL, SDA> Pins<I2c> for (SCL, SDA)
where
    SCL: PinScl<I2c>,
    SDA: PinSda<I2c>,
{
}

#[cfg(any(feature = "stm32f205", feature = "stm32f215",))]
impl PinScl<I2C1> for PB6<AlternateOD<AF4>> {}
#[cfg(any(feature = "stm32f205", feature = "stm32f215",))]
impl PinSda<I2C1> for PB7<AlternateOD<AF4>> {}

#[cfg(any(feature = "stm32f205", feature = "stm32f215",))]
impl PinScl<I2C2> for PB10<AlternateOD<AF4>> {}
#[cfg(any(feature = "stm32f205", feature = "stm32f215",))]
impl PinSda<I2C2> for PB11<AlternateOD<AF4>> {}

#[cfg(any(feature = "stm32f205", feature = "stm32f215",))]
impl PinScl<I2C3> for PA8<AlternateOD<AF4>> {}
#[cfg(any(feature = "stm32f205", feature = "stm32f215",))]
impl PinSda<I2C3> for PA9<AlternateOD<AF4>> {}

#[derive(Debug)]
pub enum Error {
    OVERRUN,
    NACK,
    TIMEOUT,
    BUS,
    CRC,
    ARBITRATION,
}

impl<PINS> I2c<I2C1, PINS> {
    pub fn i2c1(i2c: I2C1, pins: PINS, speed: KiloHertz, clocks: Clocks) -> Self
    where
        PINS: Pins<I2C1>,
    {
        unsafe {
            const EN_BIT: u8 = 21;
            const RESET_BIT: u8 = 21;
            // NOTE(unsafe) this reference will only be used for atomic writes with no side effects.
            let rcc = &(*RCC::ptr());

            // Enable and reset clock.
            bb::set(&rcc.apb1enr, EN_BIT);
            bb::set(&rcc.apb1rstr, RESET_BIT);
            bb::clear(&rcc.apb1rstr, RESET_BIT);
        }

        let i2c = I2c { i2c, pins };
        i2c.i2c_init(speed, clocks.pclk1());
        i2c
    }
}

impl<PINS> I2c<I2C2, PINS> {
    pub fn i2c2(i2c: I2C2, pins: PINS, speed: KiloHertz, clocks: Clocks) -> Self
    where
        PINS: Pins<I2C2>,
    {
        unsafe {
            const EN_BIT: u8 = 22;
            const RESET_BIT: u8 = 22;
            // NOTE(unsafe) this reference will only be used for atomic writes with no side effects.
            let rcc = &(*RCC::ptr());

            // Enable and reset clock.
            bb::set(&rcc.apb1enr, EN_BIT);
            bb::set(&rcc.apb1rstr, RESET_BIT);
            bb::clear(&rcc.apb1rstr, RESET_BIT);
        }

        let i2c = I2c { i2c, pins };
        i2c.i2c_init(speed, clocks.pclk1());
        i2c
    }
}

impl<PINS> I2c<I2C3, PINS> {
    pub fn i2c3(i2c: I2C3, pins: PINS, speed: KiloHertz, clocks: Clocks) -> Self
    where
        PINS: Pins<I2C3>,
    {
        unsafe {
            const EN_BIT: u8 = 23;
            const RESET_BIT: u8 = 23;
            // NOTE(unsafe) this reference will only be used for atomic writes with no side effects.
            let rcc = &(*RCC::ptr());

            // Enable and reset clock.
            bb::set(&rcc.apb1enr, EN_BIT);
            bb::set(&rcc.apb1rstr, RESET_BIT);
            bb::clear(&rcc.apb1rstr, RESET_BIT);
        }

        let i2c = I2c { i2c, pins };
        i2c.i2c_init(speed, clocks.pclk1());
        i2c
    }
}

impl<I2C, PINS> I2c<I2C, PINS>
where
    I2C: Deref<Target = i2c1::RegisterBlock>,
{
    fn i2c_init(&self, speed: KiloHertz, pclk: Hertz) {
        let speed: Hertz = speed.into();

        // Make sure the I2C unit is disabled so we can configure it
        self.i2c.cr1.modify(|_, w| w.pe().clear_bit());

        // Calculate settings for I2C speed modes
        let clock = pclk.0;
        let freq = clock / 1_000_000;
        assert!(freq >= 2 && freq <= 50);

        // Configure bus frequency into I2C peripheral
        self.i2c.cr2.write(|w| unsafe { w.freq().bits(freq as u8) });

        let trise = if speed <= 100.khz().into() {
            freq + 1
        } else {
            (freq * 300) / 1000 + 1
        };

        // Configure correct rise times
        self.i2c.trise.write(|w| w.trise().bits(trise as u8));

        // I2C clock control calculation
        if speed <= 100.khz().into() {
            let ccr = {
                let ccr = clock / (speed.0 * 2);
                if ccr < 4 {
                    4
                } else {
                    ccr
                }
            };

            // Set clock to standard mode with appropriate parameters for selected speed
            self.i2c.ccr.write(|w| unsafe {
                w.f_s()
                    .clear_bit()
                    .duty()
                    .clear_bit()
                    .ccr()
                    .bits(ccr as u16)
            });
        } else {
            const DUTYCYCLE: u8 = 0;
            if DUTYCYCLE == 0 {
                let ccr = clock / (speed.0 * 3);
                let ccr = if ccr < 1 { 1 } else { ccr };

                // Set clock to fast mode with appropriate parameters for selected speed (2:1 duty cycle)
                self.i2c.ccr.write(|w| unsafe {
                    w.f_s().set_bit().duty().clear_bit().ccr().bits(ccr as u16)
                });
            } else {
                let ccr = clock / (speed.0 * 25);
                let ccr = if ccr < 1 { 1 } else { ccr };

                // Set clock to fast mode with appropriate parameters for selected speed (16:9 duty cycle)
                self.i2c.ccr.write(|w| unsafe {
                    w.f_s().set_bit().duty().set_bit().ccr().bits(ccr as u16)
                });
            }
        }

        // Enable the I2C processing
        self.i2c.cr1.modify(|_, w| w.pe().set_bit());
    }

    fn check_and_clear_error_flags(&self) -> Result<i2c1::sr1::R, Error> {
        // Note that flags should only be cleared once they have been registered. If flags are
        // cleared otherwise, there may be an inherent race condition and flags may be missed.
        let sr1 = self.i2c.sr1.read();

        if sr1.timeout().bit_is_set() {
            self.i2c.sr1.modify(|_, w| w.timeout().clear_bit());
            return Err(Error::TIMEOUT);
        }

        if sr1.pecerr().bit_is_set() {
            self.i2c.sr1.modify(|_, w| w.pecerr().clear_bit());
            return Err(Error::CRC);
        }

        if sr1.ovr().bit_is_set() {
            self.i2c.sr1.modify(|_, w| w.ovr().clear_bit());
            return Err(Error::OVERRUN);
        }

        if sr1.af().bit_is_set() {
            self.i2c.sr1.modify(|_, w| w.af().clear_bit());
            return Err(Error::NACK);
        }

        if sr1.arlo().bit_is_set() {
            self.i2c.sr1.modify(|_, w| w.arlo().clear_bit());
            return Err(Error::ARBITRATION);
        }

        if sr1.berr().bit_is_set() {
            self.i2c.sr1.modify(|_, w| w.berr().clear_bit());
            return Err(Error::BUS);
        }

        Ok(sr1)
    }

    pub fn release(self) -> (I2C, PINS) {
        (self.i2c, self.pins)
    }
}

trait I2cCommon {
    fn write_bytes(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Error>;

    fn send_byte(&self, byte: u8) -> Result<(), Error>;

    fn recv_byte(&self) -> Result<u8, Error>;
}

impl<I2C, PINS> I2cCommon for I2c<I2C, PINS>
where
    I2C: Deref<Target = i2c1::RegisterBlock>,
{
    fn write_bytes(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Error> {
        // Send a START condition
        self.i2c.cr1.modify(|_, w| w.start().set_bit());

        // Wait until START condition was generated
        while self.check_and_clear_error_flags()?.sb().bit_is_clear() {}

        // Also wait until signalled we're master and everything is waiting for us
        while {
            self.check_and_clear_error_flags()?;

            let sr2 = self.i2c.sr2.read();
            sr2.msl().bit_is_clear() && sr2.busy().bit_is_clear()
        } {}

        // Set up current address, we're trying to talk to
        self.i2c
            .dr
            .write(|w| unsafe { w.bits(u32::from(addr) << 1) });

        // Wait until address was sent
        while {
            // Check for any I2C errors. If a NACK occurs, the ADDR bit will never be set.
            let sr1 = self.check_and_clear_error_flags()?;

            // Wait for the address to be acknowledged
            sr1.addr().bit_is_clear()
        } {}

        // Clear condition by reading SR2
        self.i2c.sr2.read();

        // Send bytes
        for c in bytes {
            self.send_byte(*c)?;
        }

        // Fallthrough is success
        Ok(())
    }

    fn send_byte(&self, byte: u8) -> Result<(), Error> {
        // Wait until we're ready for sending
        while {
            // Check for any I2C errors. If a NACK occurs, the ADDR bit will never be set.
            self.check_and_clear_error_flags()?.tx_e().bit_is_clear()
        } {}

        // Push out a byte of data
        self.i2c.dr.write(|w| unsafe { w.bits(u32::from(byte)) });

        // Wait until byte is transferred
        while {
            // Check for any potential error conditions.
            self.check_and_clear_error_flags()?.btf().bit_is_clear()
        } {}

        Ok(())
    }

    fn recv_byte(&self) -> Result<u8, Error> {
        while {
            // Check for any potential error conditions.
            self.check_and_clear_error_flags()?;

            self.i2c.sr1.read().rx_ne().bit_is_clear()
        } {}

        let value = self.i2c.dr.read().bits() as u8;
        Ok(value)
    }
}

impl<I2C, PINS> WriteRead for I2c<I2C, PINS>
where
    I2C: Deref<Target = i2c1::RegisterBlock>,
{
    type Error = Error;

    fn write_read(&mut self, addr: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.write_bytes(addr, bytes)?;
        self.read(addr, buffer)?;

        Ok(())
    }
}

impl<I2C, PINS> Write for I2c<I2C, PINS>
where
    I2C: Deref<Target = i2c1::RegisterBlock>,
{
    type Error = Error;

    fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.write_bytes(addr, bytes)?;

        // Send a STOP condition
        self.i2c.cr1.modify(|_, w| w.stop().set_bit());

        // Wait for STOP condition to transmit.
        while self.i2c.cr1.read().stop().bit_is_set() {}

        // Fallthrough is success
        Ok(())
    }
}

impl<I2C, PINS> Read for I2c<I2C, PINS>
where
    I2C: Deref<Target = i2c1::RegisterBlock>,
{
    type Error = Error;

    fn read(&mut self, addr: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        if let Some((last, buffer)) = buffer.split_last_mut() {
            // Send a START condition and set ACK bit
            self.i2c
                .cr1
                .modify(|_, w| w.start().set_bit().ack().set_bit());

            // Wait until START condition was generated
            while self.i2c.sr1.read().sb().bit_is_clear() {}

            // Also wait until signalled we're master and everything is waiting for us
            while {
                let sr2 = self.i2c.sr2.read();
                sr2.msl().bit_is_clear() && sr2.busy().bit_is_clear()
            } {}

            // Set up current address, we're trying to talk to
            self.i2c
                .dr
                .write(|w| unsafe { w.bits((u32::from(addr) << 1) + 1) });

            // Wait until address was sent
            while {
                self.check_and_clear_error_flags()?;
                self.i2c.sr1.read().addr().bit_is_clear()
            } {}

            // Clear condition by reading SR2
            self.i2c.sr2.read();

            // Receive bytes into buffer
            for c in buffer {
                *c = self.recv_byte()?;
            }

            // Prepare to send NACK then STOP after next byte
            self.i2c
                .cr1
                .modify(|_, w| w.ack().clear_bit().stop().set_bit());

            // Receive last byte
            *last = self.recv_byte()?;

            // Wait for the STOP to be sent.
            while self.i2c.cr1.read().stop().bit_is_set() {}

            // Fallthrough is success
            Ok(())
        } else {
            Err(Error::OVERRUN)
        }
    }
}
