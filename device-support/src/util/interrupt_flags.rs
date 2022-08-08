use core::sync::atomic::AtomicBool;

use lc3_traits::peripherals::{gpio::GpioPinArr, timers::TimerArr};

#[derive(Debug)]
pub struct PeripheralInterruptFlags {
    pub gpio: GpioPinArr<AtomicBool>, // No payload; just tell us if a rising edge has happened
    pub gpio_bank_b: GpioPinArr<AtomicBool>,
    pub gpio_bank_c: GpioPinArr<AtomicBool>,
    pub timers: TimerArr<AtomicBool>, // No payload; timers don't actually expose counts anyways
    pub input: AtomicBool, // No payload; check KBDR for the current character
    pub output: AtomicBool, // Technically this has an interrupt, but I have no idea why; UPDATE: it interrupts when it's ready to accept more data
                        // display: bool, // Unless we're exposing vsync/hsync or something, this doesn't need an interrupt
}

impl PeripheralInterruptFlags {
    pub const fn new() -> Self {
        macro_rules! b {
            () => {
                AtomicBool::new(false)
            };
        }

        // TODO: make this less gross..
        Self {
            gpio: GpioPinArr([b!(), b!(), b!(), b!(), b!(), b!(), b!(), b!()]),
            gpio_bank_b: GpioPinArr([b!(), b!(), b!(), b!(), b!(), b!(), b!(), b!()]),
            gpio_bank_c: GpioPinArr([b!(), b!(), b!(), b!(), b!(), b!(), b!(), b!()]),
            timers: TimerArr([b!(), b!()]),
            input: AtomicBool::new(false),
            output: AtomicBool::new(false),
        }
    }
}

impl Default for PeripheralInterruptFlags {
    fn default() -> Self {
        Self::new()
    }
}
