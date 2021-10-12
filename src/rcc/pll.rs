use crate::pac::RCC;

pub struct MainPll {
    pub use_pll: bool,
    pub pllsysclk: Option<u32>,
    pub pll48clk: Option<u32>,
    /// "M" divisor, required for the other PLLs on some MCUs.
    pub m: Option<u32>,
    /// "R" output, required for I2S on STM32F410.
    pub plli2sclk: Option<u32>,
}

impl MainPll {
    pub fn fast_setup(
        pllsrcclk: u32,
        use_hse: bool,
        pllsysclk: Option<u32>,
        pll48clk: bool,
    ) -> MainPll {
        let sysclk = pllsysclk.unwrap_or(pllsrcclk);
        if pllsysclk.is_none() && !pll48clk {
            // Even if we do not use the main PLL, we still need to set the PLL source as that setting
            // applies to the I2S and SAI PLLs as well.
            unsafe { &*RCC::ptr() }
                .pllcfgr
                .write(|w| w.pllsrc().bit(use_hse));

            return MainPll {
                use_pll: false,
                pllsysclk: None,
                pll48clk: None,
                m: None,
                plli2sclk: None,
            };
        }
        // Input divisor from PLL source clock, must result to frequency in
        // the range from 1 to 2 MHz
        let pllm_min = (pllsrcclk + 1_999_999) / 2_000_000;
        let pllm_max = pllsrcclk / 1_000_000;

        // Sysclk output divisor must be one of 2, 4, 6 or 8
        let sysclk_div = core::cmp::min(8, (120_000_000 / sysclk) & !1);

        let target_freq = if pll48clk {
            48_000_000
        } else {
            sysclk * sysclk_div
        };

        // Find the lowest pllm value that minimize the difference between
        // target frequency and the real vco_out frequency.
        let pllm = (pllm_min..=pllm_max)
            .min_by_key(|pllm| {
                let vco_in = pllsrcclk / pllm;
                let plln = target_freq / vco_in;
                target_freq - vco_in * plln
            })
            .unwrap();

        let vco_in = pllsrcclk / pllm;
        assert!((1_000_000..=2_000_000).contains(&vco_in));

        // Main scaler, must result in >= 100MHz (>= 192MHz for F401)
        // and <= 432MHz, min 50, max 432
        let plln = if pll48clk {
            // try the different valid pllq according to the valid
            // main scaller values, and take the best
            let pllq = (4..=9)
                .min_by_key(|pllq| {
                    let plln = 48_000_000 * pllq / vco_in;
                    let pll48_diff = 48_000_000 - vco_in * plln / pllq;
                    let sysclk_diff = (sysclk as i32 - (vco_in * plln / sysclk_div) as i32).abs();
                    (pll48_diff, sysclk_diff)
                })
                .unwrap();
            48_000_000 * pllq / vco_in
        } else {
            192 // FIXME
                // sysclk * sysclk_div / vco_in
        };
        assert!((192..=432).contains(&plln));

        let vco_out = pllsrcclk * (plln / pllm);
        // PLLP: Main PLL (PLL) division factor for main system clock
        // Caution: The software has to set these bits correctly not to exceed 120 MHz on this domain.
        let pllp = vco_out / 120_000_000;
        assert!((0..=3).contains(&pllp));

        let pllq = vco_out / 48_000_000;
        assert!((2..=15).contains(&pllq));
        //let pllq = (vco_in * plln + 47_999_999) / 48_000_000;
        //let real_pll48clk = vco_in * plln / pllq;

        unsafe { &*RCC::ptr() }.pllcfgr.write(|w| unsafe {
            w.pllm().bits(pllm as u8);
            w.plln().bits(plln as u16);
            w.pllp().bits(pllp as u8);
            w.pllq().bits(pllq as u8);
            w.pllsrc().bit(use_hse)
        });

        let real_pllsysclk = vco_in * plln / sysclk_div;

        MainPll {
            use_pll: true,
            pllsysclk: Some(real_pllsysclk),
            pll48clk: None,
            m: Some(pllm),
            plli2sclk: None,
        }
    }
}

pub struct I2sPll {
    pub use_pll: bool,
    /// "M" divisor, required for the other PLLs on some MCUs.
    pub m: Option<u32>,
    /// PLL I2S clock output.
    pub plli2sclk: Option<u32>,
}

impl I2sPll {
    pub fn unused() -> I2sPll {
        I2sPll {
            use_pll: false,
            m: None,
            plli2sclk: None,
        }
    }

    pub fn setup(pllsrcclk: u32, plli2sclk: Option<u32>) -> I2sPll {
        let target = if let Some(clk) = plli2sclk {
            clk
        } else {
            return Self::unused();
        };
        // Input divisor from PLL source clock, must result to frequency in
        // the range from 1 to 2 MHz
        let pllm_min = (pllsrcclk + 1_999_999) / 2_000_000;
        let pllm_max = pllsrcclk / 1_000_000;
        let (pll, config, _) = (pllm_min..=pllm_max)
            .map(|m| Self::optimize_fixed_m(pllsrcclk, m, target))
            .min_by_key(|(_, _, error)| *error)
            .expect("no suitable I2S PLL configuration found");
        Self::apply_config(config);
        pll
    }

    pub fn setup_shared_m(pllsrcclk: u32, m: Option<u32>, plli2sclk: Option<u32>) -> I2sPll {
        // "m" is None if the main PLL is not in use.
        let m = if let Some(m) = m {
            m
        } else {
            return Self::setup(pllsrcclk, plli2sclk);
        };
        let target = if let Some(clk) = plli2sclk {
            clk
        } else {
            return Self::unused();
        };
        let (pll, config, _) = Self::optimize_fixed_m(pllsrcclk, m, target);
        Self::apply_config(config);
        pll
    }

    fn optimize_fixed_m(pllsrcclk: u32, m: u32, plli2sclk: u32) -> (I2sPll, SingleOutputPll, u32) {
        let (config, real_plli2sclk, error) =
            SingleOutputPll::optimize(pllsrcclk, m, plli2sclk, 2, 7)
                .expect("did not find any valid I2S PLL config");
        (
            I2sPll {
                use_pll: true,
                m: Some(config.m as u32),
                plli2sclk: Some(real_plli2sclk),
            },
            config,
            error,
        )
    }

    fn apply_config(config: SingleOutputPll) {
        let rcc = unsafe { &*RCC::ptr() };
        // "M" may have been written before, but the value is identical.
        rcc.pllcfgr
            .modify(|_, w| unsafe { w.pllm().bits(config.m) });
        rcc.plli2scfgr
            .modify(|_, w| unsafe { w.plli2sn().bits(config.n).plli2sr().bits(config.outdiv) });
    }
}

struct SingleOutputPll {
    m: u8,
    n: u16,
    outdiv: u8,
}

impl SingleOutputPll {
    fn optimize(
        pllsrcclk: u32,
        m: u32,
        target: u32,
        min_div: u32,
        max_div: u32,
    ) -> Option<(SingleOutputPll, u32, u32)> {
        let vco_in = pllsrcclk / m;

        // We loop through the possible divider values to find the best configuration. Looping
        // through all possible "N" values would result in more iterations.
        let (n, outdiv, output, error) = (min_div..=max_div)
            .filter_map(|outdiv| {
                let target_vco_out = target * outdiv;
                let n = (target_vco_out + (vco_in >> 1)) / vco_in;
                let vco_out = vco_in * n;
                if !(100_000_000..=432_000_000).contains(&vco_out) {
                    return None;
                }
                let output = vco_out / outdiv;
                let error = (output as i32 - target as i32).abs() as u32;
                Some((n, outdiv, output, error))
            })
            .min_by_key(|(_, _, _, error)| *error)?;
        Some((
            SingleOutputPll {
                m: m as u8,
                n: n as u16,
                outdiv: outdiv as u8,
            },
            output,
            error,
        ))
    }
}
