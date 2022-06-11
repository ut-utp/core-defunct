//! TODO!

use lc3_isa::{MEM_MAPPED_START_ADDR, INTERRUPT_SERVICE_ROUTINES_START_ADDR};
use lc3_traits::peripherals::gpio::GpioBank as Gp;

// TODO: split into modules/structs
pub const KBSR_ADDR: Addr = 0xFE00;
pub const KBDR_ADDR: Addr = 0xFE02;

pub const KEYBOARD_INT_VEC: u8 = 0x80;
pub const KEYBOARD_INT_PRIORITY: u8 = 4;

pub const DSR_ADDR: Addr = 0xFE04;
pub const DDR_ADDR: Addr = 0xFE06;

pub const DISPLAY_INT_VEC: u8 = 0x81; // TODO: What is this actually?
pub const DISPLAY_INT_PRIORITY: u8 = 4;

pub const GPIO_OFFSET: u8 = 0x10;
const GPIO_MEM_MAPPED_BASE: Addr = MEM_MAPPED_START_ADDR + (GPIO_OFFSET as Addr);
const GPIO_ADDRS_PER_PIN: Addr = 2;

pub const GA0_CR_ADDR: Addr = gpio_cr_addr(Gp::A, 0); // xFE10
pub const GA0_DR_ADDR: Addr = gpio_dr_addr(Gp::A, 0); // xFE11
pub const GA1_CR_ADDR: Addr = gpio_cr_addr(Gp::A, 1); // xFE12
pub const GA1_DR_ADDR: Addr = gpio_dr_addr(Gp::A, 1); // xFE13
pub const GA2_CR_ADDR: Addr = gpio_cr_addr(Gp::A, 2); // xFE14
pub const GA2_DR_ADDR: Addr = gpio_dr_addr(Gp::A, 2); // xFE15
pub const GA3_CR_ADDR: Addr = gpio_cr_addr(Gp::A, 3); // xFE16
pub const GA3_DR_ADDR: Addr = gpio_dr_addr(Gp::A, 3); // xFE17
pub const GA4_CR_ADDR: Addr = gpio_cr_addr(Gp::A, 4); // xFE18
pub const GA4_DR_ADDR: Addr = gpio_dr_addr(Gp::A, 4); // xFE19
pub const GA5_CR_ADDR: Addr = gpio_cr_addr(Gp::A, 5); // xFE1A
pub const GA5_DR_ADDR: Addr = gpio_dr_addr(Gp::A, 5); // xFE1B
pub const GA6_CR_ADDR: Addr = gpio_cr_addr(Gp::A, 6); // xFE1C
pub const GA6_DR_ADDR: Addr = gpio_dr_addr(Gp::A, 6); // xFE1D
pub const GA7_CR_ADDR: Addr = gpio_cr_addr(Gp::A, 7); // xFE1E
pub const GA7_DR_ADDR: Addr = gpio_dr_addr(Gp::A, 7); // xFE1F

pub const GB0_CR_ADDR: Addr = gpio_cr_addr(Gp::B, 0); // xFE20
pub const GB0_DR_ADDR: Addr = gpio_dr_addr(Gp::B, 0); // xFE21
pub const GB1_CR_ADDR: Addr = gpio_cr_addr(Gp::B, 1); // xFE22
pub const GB1_DR_ADDR: Addr = gpio_dr_addr(Gp::B, 1); // xFE23
pub const GB2_CR_ADDR: Addr = gpio_cr_addr(Gp::B, 2); // xFE24
pub const GB2_DR_ADDR: Addr = gpio_dr_addr(Gp::B, 2); // xFE25
pub const GB3_CR_ADDR: Addr = gpio_cr_addr(Gp::B, 3); // xFE26
pub const GB3_DR_ADDR: Addr = gpio_dr_addr(Gp::B, 3); // xFE27
pub const GB4_CR_ADDR: Addr = gpio_cr_addr(Gp::B, 4); // xFE28
pub const GB4_DR_ADDR: Addr = gpio_dr_addr(Gp::B, 4); // xFE29
pub const GB5_CR_ADDR: Addr = gpio_cr_addr(Gp::B, 5); // xFE2A
pub const GB5_DR_ADDR: Addr = gpio_dr_addr(Gp::B, 5); // xFE2B
pub const GB6_CR_ADDR: Addr = gpio_cr_addr(Gp::B, 6); // xFE2C
pub const GB6_DR_ADDR: Addr = gpio_dr_addr(Gp::B, 6); // xFE2D
pub const GB7_CR_ADDR: Addr = gpio_cr_addr(Gp::B, 7); // xFE2E
pub const GB7_DR_ADDR: Addr = gpio_dr_addr(Gp::B, 7); // xFE2F

pub const GC0_CR_ADDR: Addr = gpio_cr_addr(Gp::C, 0); // xFE30
pub const GC0_DR_ADDR: Addr = gpio_dr_addr(Gp::C, 0); // xFE31
pub const GC1_CR_ADDR: Addr = gpio_cr_addr(Gp::C, 1); // xFE32
pub const GC1_DR_ADDR: Addr = gpio_dr_addr(Gp::C, 1); // xFE33
pub const GC2_CR_ADDR: Addr = gpio_cr_addr(Gp::C, 2); // xFE34
pub const GC2_DR_ADDR: Addr = gpio_dr_addr(Gp::C, 2); // xFE35
pub const GC3_CR_ADDR: Addr = gpio_cr_addr(Gp::C, 3); // xFE36
pub const GC3_DR_ADDR: Addr = gpio_dr_addr(Gp::C, 3); // xFE37
pub const GC4_CR_ADDR: Addr = gpio_cr_addr(Gp::C, 4); // xFE38
pub const GC4_DR_ADDR: Addr = gpio_dr_addr(Gp::C, 4); // xFE39
pub const GC5_CR_ADDR: Addr = gpio_cr_addr(Gp::C, 5); // xFE3A
pub const GC5_DR_ADDR: Addr = gpio_dr_addr(Gp::C, 5); // xFE3B
pub const GC6_CR_ADDR: Addr = gpio_cr_addr(Gp::C, 6); // xFE3C
pub const GC6_DR_ADDR: Addr = gpio_dr_addr(Gp::C, 6); // xFE3D
pub const GC7_CR_ADDR: Addr = gpio_cr_addr(Gp::C, 7); // xFE3E
pub const GC7_DR_ADDR: Addr = gpio_dr_addr(Gp::C, 7); // xFE3F


const fn gpio_cr_addr(bank: GpioBank, i: u16) -> Addr {
    GPIO_MEM_MAPPED_BASE +
        ((GpioPin::NUM_PINS as Word) * GPIO_ADDRS_PER_PIN * (bank as Word)) +
        GPIO_ADDRS_PER_PIN * i
}
const fn gpio_dr_addr(bank: GpioBank, i: u16) -> Addr {
    gpio_cr_addr(bank, i) + 1
}

// TODO: rename
pub const GPIO_A_CR_ADDR: Addr = 0xFE08;
pub const GPIO_A_DR_ADDR: Addr = 0xFE09;
pub const GPIO_B_CR_ADDR: Addr = 0xFE0A;
pub const GPIO_B_DR_ADDR: Addr = 0xFE0B;
pub const GPIO_C_CR_ADDR: Addr = 0xFE0C;
pub const GPIO_C_DR_ADDR: Addr = 0xFE0D;

pub const GPIO_BASE_INT_VEC: Addr = INTERRUPT_SERVICE_ROUTINES_START_ADDR + (GPIO_OFFSET as Addr); // x190
pub const GA0_INT_VEC: u8 = 128 + GPIO_OFFSET + 8 * 0 + 0; // x90
pub const GA1_INT_VEC: u8 = 128 + GPIO_OFFSET + 8 * 0 + 1; // x91
pub const GA2_INT_VEC: u8 = 128 + GPIO_OFFSET + 8 * 0 + 2; // x92
pub const GA3_INT_VEC: u8 = 128 + GPIO_OFFSET + 8 * 0 + 3; // x93
pub const GA4_INT_VEC: u8 = 128 + GPIO_OFFSET + 8 * 0 + 4; // x94
pub const GA5_INT_VEC: u8 = 128 + GPIO_OFFSET + 8 * 0 + 5; // x95
pub const GA6_INT_VEC: u8 = 128 + GPIO_OFFSET + 8 * 0 + 6; // x96
pub const GA7_INT_VEC: u8 = 128 + GPIO_OFFSET + 8 * 0 + 7; // x97

pub const GB0_INT_VEC: u8 = 128 + GPIO_OFFSET + 8 * 1 + 0; // x98
pub const GB1_INT_VEC: u8 = 128 + GPIO_OFFSET + 8 * 1 + 1; // x99
pub const GB2_INT_VEC: u8 = 128 + GPIO_OFFSET + 8 * 1 + 2; // x9A
pub const GB3_INT_VEC: u8 = 128 + GPIO_OFFSET + 8 * 1 + 3; // x9B
pub const GB4_INT_VEC: u8 = 128 + GPIO_OFFSET + 8 * 1 + 4; // x9C
pub const GB5_INT_VEC: u8 = 128 + GPIO_OFFSET + 8 * 1 + 5; // x9D
pub const GB6_INT_VEC: u8 = 128 + GPIO_OFFSET + 8 * 1 + 6; // x9E
pub const GB7_INT_VEC: u8 = 128 + GPIO_OFFSET + 8 * 1 + 7; // x9F

pub const GC0_INT_VEC: u8 = 128 + GPIO_OFFSET + 8 * 2 + 0; // xA0
pub const GC1_INT_VEC: u8 = 128 + GPIO_OFFSET + 8 * 2 + 1; // xA1
pub const GC2_INT_VEC: u8 = 128 + GPIO_OFFSET + 8 * 2 + 2; // xA2
pub const GC3_INT_VEC: u8 = 128 + GPIO_OFFSET + 8 * 2 + 3; // xA3
pub const GC4_INT_VEC: u8 = 128 + GPIO_OFFSET + 8 * 2 + 4; // xA4
pub const GC5_INT_VEC: u8 = 128 + GPIO_OFFSET + 8 * 2 + 5; // xA5
pub const GC6_INT_VEC: u8 = 128 + GPIO_OFFSET + 8 * 2 + 6; // xA6
pub const GC7_INT_VEC: u8 = 128 + GPIO_OFFSET + 8 * 2 + 7; // xA7

pub const GPIO_INT_PRIORITY: u8 = 4;

pub const ADC_OFFSET: u8 = 0x40;
const ADC_MEM_MAPPED_BASE: Addr = MEM_MAPPED_START_ADDR + (ADC_OFFSET as Addr);
const ADC_PIN_ADDRS: Addr = 2;

pub const A0CR_ADDR: Addr = ADC_MEM_MAPPED_BASE + ADC_PIN_ADDRS * 0 + 0; // xFE40
pub const A0DR_ADDR: Addr = ADC_MEM_MAPPED_BASE + ADC_PIN_ADDRS * 0 + 1; // xFE41
pub const A1CR_ADDR: Addr = ADC_MEM_MAPPED_BASE + ADC_PIN_ADDRS * 1 + 0; // xFE42
pub const A1DR_ADDR: Addr = ADC_MEM_MAPPED_BASE + ADC_PIN_ADDRS * 1 + 1; // xFE43
pub const A2CR_ADDR: Addr = ADC_MEM_MAPPED_BASE + ADC_PIN_ADDRS * 2 + 0; // xFE44
pub const A2DR_ADDR: Addr = ADC_MEM_MAPPED_BASE + ADC_PIN_ADDRS * 2 + 1; // xFE45
pub const A3CR_ADDR: Addr = ADC_MEM_MAPPED_BASE + ADC_PIN_ADDRS * 3 + 0; // xFE46
pub const A3DR_ADDR: Addr = ADC_MEM_MAPPED_BASE + ADC_PIN_ADDRS * 3 + 1; // xFE47
pub const A4CR_ADDR: Addr = ADC_MEM_MAPPED_BASE + ADC_PIN_ADDRS * 4 + 0; // xFE48
pub const A4DR_ADDR: Addr = ADC_MEM_MAPPED_BASE + ADC_PIN_ADDRS * 4 + 1; // xFE49
pub const A5CR_ADDR: Addr = ADC_MEM_MAPPED_BASE + ADC_PIN_ADDRS * 5 + 0; // xFE4A
pub const A5DR_ADDR: Addr = ADC_MEM_MAPPED_BASE + ADC_PIN_ADDRS * 5 + 1; // xFE4B

pub const PWM_OFFSET: u8 = 0x50;
const PWM_MEM_MAPPED_BASE: Addr = MEM_MAPPED_START_ADDR + (PWM_OFFSET as Addr);
const PWM_PIN_ADDRS: Addr = 2;

pub const P0CR_ADDR: Addr = PWM_MEM_MAPPED_BASE + PWM_PIN_ADDRS * 0 + 0; // xFE50
pub const P0DR_ADDR: Addr = PWM_MEM_MAPPED_BASE + PWM_PIN_ADDRS * 0 + 1; // xFE51
pub const P1CR_ADDR: Addr = PWM_MEM_MAPPED_BASE + PWM_PIN_ADDRS * 1 + 0; // xFE52
pub const P1DR_ADDR: Addr = PWM_MEM_MAPPED_BASE + PWM_PIN_ADDRS * 1 + 1; // xFE53

pub const TIMER_OFFSET: u8 = 0x60;
const TIMER_MEM_MAPPED_BASE: Addr = MEM_MAPPED_START_ADDR + (TIMER_OFFSET as Addr);
const TIMER_PIN_ADDRS: Addr = 2;

pub const T0CR_ADDR: Addr = TIMER_MEM_MAPPED_BASE + TIMER_PIN_ADDRS * 0 + 0; // xFE60
pub const T0DR_ADDR: Addr = TIMER_MEM_MAPPED_BASE + TIMER_PIN_ADDRS * 0 + 1; // xFE61
pub const T1CR_ADDR: Addr = TIMER_MEM_MAPPED_BASE + TIMER_PIN_ADDRS * 1 + 0; // xFE62
pub const T1DR_ADDR: Addr = TIMER_MEM_MAPPED_BASE + TIMER_PIN_ADDRS * 1 + 1; // xFE63

pub const TIMER_BASE_INT_VEC: Addr = INTERRUPT_SERVICE_ROUTINES_START_ADDR + (TIMER_OFFSET as Addr); // x1E0;       // TODO: do this in a better way
pub const T0_INT_VEC: u8 = 128 + TIMER_OFFSET + 0; // xE0
pub const T1_INT_VEC: u8 = 128 + TIMER_OFFSET + 1; // xE1;
pub const TIMER_INT_PRIORITY: u8 = 4;

// (For one off peripherals like the clock and the display, etc.)
pub const MISC_OFFSET: u8 = 0x70;
const MISC_MEM_MAPPED_BASE: Addr = MEM_MAPPED_START_ADDR + (MISC_OFFSET as Addr);

pub const CLKR_ADDR: Addr = MISC_MEM_MAPPED_BASE + 0; // xFE70

pub const OPT_PERI_REG_ADDR: Addr = MISC_MEM_MAPPED_BASE + 1; // xFE71 // TODO! use `get_all_optional_peripherals_status`! 1 bit per optional peripheral!
                                                                       // maybe also expose traps for each? or at least 1 trap for GET_SUPPORTED_PERIPHERALS

pub const BSP_ADDR: Addr = 0xFFFA;

use crate::interp::{InstructionInterpreterPeripheralAccess, DerefsIntoPeripheralsWrapper};
use crate::interp::InstructionInterpreterPeripheralAccess as Ipa;
use core::ops::Deref;
use lc3_isa::{Addr, Bits, SignedWord, Word, MCR as MCR_ADDR, PSR as PSR_ADDR, WORD_MAX_VAL};
use lc3_traits::peripherals::Peripherals;
use lc3_traits::error::Error;

use crate::interp::{Acv, WriteAttempt};

pub trait MemMapped: Deref<Target = Word> + Sized {
    const ADDR: Addr;
    const HAS_STATEFUL_READS: bool = false;

    fn with_value(value: Word) -> Self;

    fn from<I: InstructionInterpreterPeripheralAccess>(interp: &I) -> Result<Self, Acv> {
        // Checked access by default:
        Ok(Self::with_value(interp.get_word(Self::ADDR)?))
    }

    fn set<I: InstructionInterpreterPeripheralAccess>(interp: &mut I, value: Word) -> WriteAttempt {
        // Checked access by default:
        interp.set_word(Self::ADDR, value)
    }

    fn update<I: InstructionInterpreterPeripheralAccess>(interp: &mut I, func: impl FnOnce(Self) -> Word) -> WriteAttempt {
        Self::set(interp, func(Self::from(interp)?))
    }

    #[doc(hidden)]
    fn write_current_value<I: InstructionInterpreterPeripheralAccess>(&self, interp: &mut I) -> WriteAttempt {
        Self::set(interp, **self)
    }
}

// Don't implement this manually; you could mess up. (only implement this if
// you've overriden the default impls for from and set in the trait above).
//
// Use the macro below instead.
pub trait MemMappedSpecial: MemMapped {
    // Infallible.
    fn from_special<I: InstructionInterpreterPeripheralAccess>(interp: &I) -> Self {
        Self::from(interp).unwrap()
    }

    // Also infallible.
    fn set_special<I: InstructionInterpreterPeripheralAccess>(interp: &mut I, value: Word) {
        Self::set(interp, value).unwrap()
    }
}

pub trait Interrupt: MemMapped {
    const INT_VEC: u8;
    const PRIORITY: u8; // Must be a 3 bit number

    /// Returns true if:
    ///   - this particular interrupt is enabled
    ///   - this particular interrupt is ready to fire (i.e. pending).
    fn interrupt<I: InstructionInterpreterPeripheralAccess>(interp: &I) -> bool {
        // TODO: this is not true anymore, verify
        // Important that interrupt_enabled is first: we do want to short circuit here!
        Self::interrupt_enabled(interp) && Self::interrupt_ready(interp)
    }

    // TODO: eventually, const
    fn priority() -> u8 {
        (Self::PRIORITY as Word).u16(0..2) as u8
    }

    /// Returns true if the interrupt is ready to fire (i.e. pending) regardless
    /// of whether the interrupt is enabled.
    fn interrupt_ready<I: InstructionInterpreterPeripheralAccess>(interp: &I) -> bool;

    /// Returns true if the interrupt is enabled.
    fn interrupt_enabled<I: InstructionInterpreterPeripheralAccess>(interp: &I) -> bool;

    fn reset_interrupt_flag<I: InstructionInterpreterPeripheralAccess>(interp: &mut I);
}

// struct KBSR(Word);

// impl Deref for KBSR {
//     type Target = Word;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl MemMapped for KBSR {
//     const ADDR: Addr = 0xFE00;

//     fn with_value(value: Word) -> Self {
//         Self(value)
//     }
// }

macro_rules! mem_mapped {
    ($name:ident, $address:expr, $comment:literal) => {
        mem_mapped!(-; $name, $address, $comment);
    };

    (special: $name:ident, $address:expr, $comment:literal) => {
        mem_mapped!(+; $name, $address, $comment, "\nDoes not produce access control violations (ACVs) when accessed!");

        impl MemMappedSpecial for $name { }
    };

    ($special:tt; $name:ident, $address:expr, $comment:literal $(, $extra_comment:literal)?) => {
        #[doc=$comment]
        $(#[doc=$extra_comment])?
        #[derive(Copy, Clone, Debug, PartialEq)]
        pub struct $name(Word);

        impl Deref for $name {
            type Target = Word;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl MemMapped for $name {
            const ADDR: Addr = $address;

            fn with_value(value: Word) -> Self {
                Self(value)
            }

            _mem_mapped_special_access!($special);
        }
    }
}

macro_rules! _mem_mapped_special_access {
    (+) => {
        fn from<I: Ipa>(interp: &I) -> Result<Self, Acv> {
            // Special unchecked access!
            Ok(Self::with_value(
                interp.get_word_force_memory_backed(Self::ADDR),
            ))
        }

        fn set<I: Ipa>(
            interp: &mut I,
            value: Word,
        ) -> WriteAttempt {
            // Special unchecked access!
            interp.set_word_force_memory_backed(Self::ADDR, value);
            Ok(())
        }
    };
    (-) => {};
}

// struct KBSR(Word);

// impl Deref for KBSR {
//     type Target = Word;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl MemMapped for KBSR {
//     const ADDR: Addr = 0xFE00;

//     fn with_value(value: Word) -> Self {
//         Self(value)
//     }
// }

use lc3_traits::peripherals::input::Input;
#[doc = "Keyboard Data Register"]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct KBDR(Word);
impl Deref for KBDR {
    type Target = Word;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl MemMapped for KBDR {
    const ADDR: Addr = KBDR_ADDR;
    const HAS_STATEFUL_READS: bool = true;

    fn with_value(value: Word) -> Self {
        Self(value)
    }

    fn from<I: Ipa>(interp: &I) -> Result<Self, Acv> {
        let data = match Input::read_data(interp.get_input()) {
            Ok(val) => val as Word,
            Err(err) => {
                interp.set_error(Error::from(err));
                0
            }
        };

        Ok(Self::with_value(data))
    }

    fn set<I: Ipa>(_interp: &mut I, _value: Word) -> WriteAttempt {
        Ok(()) // TODO: Ignore writes to keyboard data register?
    }
}
// mem_mapped!(special: KBSR, 0xFE00, "Keyboard Status Register.");
// mem_mapped!(special: KBDR, 0xFE02, "Keyboard Data Register.");

/// Keyboard Status Register
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct KBSR(Word);

impl Deref for KBSR {
    type Target = Word;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MemMapped for KBSR {
    const ADDR: Addr = KBSR_ADDR;

    fn with_value(value: Word) -> Self {
        Self(value)
    }

    fn from<I: Ipa>(interp: &I) -> Result<Self, Acv> {
        // Bit 15: Ready
        // Bit 14: Interrupt Enabled
        let word = ((Input::current_data_unread(interp.get_input()) as Word) << 15)
            | ((Input::interrupts_enabled(interp.get_input()) as Word) << 14);

        Ok(Self::with_value(word))
    }

    fn set<I: Ipa>(interp: &mut I, value: Word) -> WriteAttempt {
        // Bit 15: Ready
        // Bit 14: Interrupt Enabled
        let interrupt_enabled_bit = value.bit(14);

        Input::set_interrupt_enable_bit(interp.get_input_mut(), interrupt_enabled_bit); // TODO: do something on error

        Ok(())
    }
}

impl Interrupt for KBSR {
    const INT_VEC: u8 = KEYBOARD_INT_VEC;
    const PRIORITY: u8 = KEYBOARD_INT_PRIORITY;

    fn interrupt_ready<I: Ipa>(interp: &I) -> bool {
        Input::interrupt_occurred(interp.get_input())
    }

    fn interrupt_enabled<I: Ipa>(interp: &I) -> bool {
        Input::interrupts_enabled(interp.get_input())
    }

    fn reset_interrupt_flag<I: Ipa>(interp: &mut I) {
        if Input::interrupts_enabled(interp.get_input()) {
            Input::reset_interrupt_flag(interp.get_input_mut());
        }
    }
}

// impl KBSR {
//     pub fn
// }

// TODO! these aren't special! this is a stopgap so we don't stack overflow!

use lc3_traits::peripherals::output::Output;

#[doc = "Display Status Register"]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DSR(Word);
impl Deref for DSR {
    type Target = Word;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl MemMapped for DSR {
    const ADDR: Addr = DSR_ADDR;

    fn with_value(value: Word) -> Self {
        Self(value)
    }

    fn from<I: Ipa>(interp: &I) -> Result<Self, Acv> {
        Ok(Self::with_value(
            (Output::current_data_written(interp.get_output()) as Word) << 15,
        ))
    }

    fn set<I: Ipa>(interp: &mut I, value: Word) -> WriteAttempt {
        Output::set_interrupt_enable_bit(interp.get_output_mut(), value.bit(1));
        Ok(())
    }
}

impl Interrupt for DSR {
    const INT_VEC: u8 = DISPLAY_INT_VEC;
    const PRIORITY: u8 = DISPLAY_INT_PRIORITY;

    fn interrupt_ready<I: Ipa>(interp: &I) -> bool {
        Output::interrupt_occurred(interp.get_output())
    }

    fn interrupt_enabled<I: Ipa>(interp: &I) -> bool {
        Output::interrupts_enabled(interp.get_output())
    }

    fn reset_interrupt_flag<I: Ipa>(interp: &mut I) {
        if Output::interrupts_enabled(interp.get_output()) {
            Output::reset_interrupt_flag(interp.get_output_mut());
        }
    }
}

#[doc = "Display Data Register"]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DDR(Word);
impl Deref for DDR {
    type Target = Word;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl MemMapped for DDR {
    const ADDR: Addr = DDR_ADDR;

    fn with_value(value: Word) -> Self {
        Self(value)
    }

    fn from<I: Ipa>(_interp: &I) -> Result<Self, Acv> {
        // TODO: error here?
        Ok(Self::with_value(0 as Word))
    }

    fn set<I: Ipa>(interp: &mut I, value: Word) -> WriteAttempt {
        // TODO: propagate errors here!
        let _ = Output::write_data(interp.get_output_mut(), value as u8);
        Ok(())
    }
}

trait GetGpioBank<I: Ipa> {
    type Bank: Gpio + ?Sized;
    const BANK: GpioBank;

    fn get(interp: &I) -> Option<&Self::Bank>;
    fn get_mut(interp: &mut I) -> Option<&mut Self::Bank>;
}

struct Banked<const BANK: char>;
impl<I: Ipa> GetGpioBank<I> for Banked<'A'> {
    type Bank = <<I as DerefsIntoPeripheralsWrapper>::P as Peripherals>::Gpio;
    const BANK: GpioBank = Gp::A;

    fn get(interp: &I) -> Option<&Self::Bank> { Some(interp.get_gpio()) }
    fn get_mut(interp: &mut I) -> Option<&mut Self::Bank> { Some(interp.get_gpio_mut()) }
}
impl<I: Ipa> GetGpioBank<I> for Banked<'B'> {
    type Bank = <<I as DerefsIntoPeripheralsWrapper>::P as Peripherals>::GpioB;
    const BANK: GpioBank = Gp::B;

    fn get(interp: &I) -> Option<&Self::Bank> { interp.get_gpio_bank_b() }
    fn get_mut(interp: &mut I) -> Option<&mut Self::Bank> { interp.get_gpio_bank_b_mut() }
}
impl<I: Ipa> GetGpioBank<I> for Banked<'C'> {
    type Bank = <<I as DerefsIntoPeripheralsWrapper>::P as Peripherals>::GpioC;
    const BANK: GpioBank = Gp::C;

    fn get(interp: &I) -> Option<&Self::Bank> { interp.get_gpio_bank_c() }
    fn get_mut(interp: &mut I) -> Option<&mut Self::Bank> { interp.get_gpio_bank_c_mut() }
}

fn get_gpio<const B: char, const SET_ERR: bool/*  = false */, I: Ipa>(interp: &I) -> Option<&<Banked<B> as GetGpioBank<I>>::Bank>
where
    Banked<B>: GetGpioBank<I>,
{
    let res = Banked::<B>::get(interp);
    if res.is_none() && SET_ERR {
        interp.set_error(Error::OptionalPeripheralIsNotPresent(<Banked<B>>::BANK.try_into().unwrap()));
    }

    res
}

fn get_gpio_mut<const B: char, const SET_ERR: bool/*  = false */, I: Ipa>(interp: &mut I) -> Option<&mut <Banked<B> as GetGpioBank<I>>::Bank>
where
    Banked<B>: GetGpioBank<I>,
{
    if Banked::<B>::get_mut(interp).is_none() && SET_ERR {
        interp.set_error(Error::OptionalPeripheralIsNotPresent(<Banked<B>>::BANK.try_into().unwrap()));
    }

    Banked::<B>::get_mut(interp)
}


macro_rules! gpio_mem_mapped {
    ($pin:expr, $bank:expr, $pin_name:literal, $cr:ident, $dr:ident, $cr_addr:expr, $dr_addr:expr, $int_vec:expr) => {
        #[doc=$pin_name]
        #[doc="GPIO Pin Control Register"] // TODO: format correctly
        #[derive(Copy, Clone, Debug, PartialEq)]
        pub struct $cr(Word);

        impl Deref for $cr {
            type Target = Word;

            fn deref(&self) -> &Self::Target { &self.0 }
        }

        impl MemMapped for $cr {
            const ADDR: Addr = $cr_addr;

            fn with_value(value: Word) -> Self { Self(value) }

            fn from<I: Ipa> (interp: &I) -> Result<Self, Acv> {
                if let Some(gp) = get_gpio::<$bank, true, I>(interp) {
                    let state = Gpio::get_state(gp, $pin);

                    use lc3_traits::peripherals::gpio::GpioState::*;
                    let word: Word = match state {
                        Disabled => 0,
                        Output => 1,
                        Input => 2,
                        Interrupt => 3,
                    };

                    Ok(Self::with_value(word))
                } else {
                    Ok(Self::with_value(0)) // TODO: ACV?
                }
            }

            fn set<I: Ipa>(interp: &mut I, value: Word) -> WriteAttempt {
                if let Some(gp) = get_gpio_mut::<$bank, true, I>(interp) {
                    use lc3_traits::peripherals::gpio::GpioState::*;
                    let state = match value.bits(0..1) {
                        0 => Disabled,
                        1 => Output,
                        2 => Input,
                        3 => Interrupt,
                        _ => unreachable!()
                    };

                    match Gpio::set_state(gp, $pin, state) {
                        Ok(()) => Ok(()),
                        Err(err) => {
                            interp.set_error(Error::from(err));
                            Ok(())
                        }
                    }
                } else {
                    Ok(()) // TODO: ACV?
                }

            }
        }

        impl Interrupt for $cr {
            const INT_VEC: u8 = $int_vec;
            const PRIORITY: u8 = GPIO_INT_PRIORITY;

            fn interrupt_ready<I: Ipa>(interp: &I) -> bool {
                if let Some(gp) = get_gpio::<$bank, false, I>(interp) {
                    Gpio::interrupt_occurred(gp, $pin)
                } else {
                    false
                }
            }

            fn interrupt_enabled<I: Ipa>(interp: &I) -> bool {
                if let Some(gp) = get_gpio::<$bank, false, I>(interp) {
                    Gpio::interrupts_enabled(gp, $pin)
                } else {
                    false
                }
            }

            fn reset_interrupt_flag<I: Ipa>(interp: &mut I) {
                if let Some(gp) = get_gpio_mut::<$bank, false, I>(interp) {
                    if Gpio::interrupts_enabled(gp, $pin) {
                        Gpio::reset_interrupt_flag(gp, $pin);
                    }
                }
            }

        }

        #[doc=$pin_name]
        #[doc="GPIO Pin Data Register"] // TODO: format correctly
        #[derive(Copy, Clone, Debug, PartialEq)]
        pub struct $dr(Word);

        impl Deref for $dr {
            type Target = Word;

            fn deref(&self) -> &Self::Target { &self.0 }
        }

        impl MemMapped for $dr {
            const ADDR: Addr = $dr_addr;

            fn with_value(value: Word) -> Self { Self(value) }

            // TODO: change all these to some other kind of error since we already check for ACVs in read_word, etc.
            fn from<I: Ipa>(interp: &I) -> Result<Self, Acv> {
                if let Some(gp) = get_gpio::<$bank, true, I>(interp) {
                    let word = match Gpio::read(gp, $pin) {
                        Ok(val) => val as Word,
                        Err(err) => {
                            interp.set_error(Error::from(err));
                            0x8000
                        }
                    };

                    Ok(Self::with_value(word))
                } else {
                    Ok(Self::with_value(0x8000)) // TODO: ACV?
                }
            }

            fn set<I: Ipa>(interp: &mut I, value: Word) -> WriteAttempt {
                if let Some(gp) = get_gpio_mut::<$bank, true, I>(interp) {
                    let bit: bool = value.bit(0);
                    match Gpio::write(gp, $pin, bit) {
                        Ok(()) => Ok(()),
                        Err(err) => {
                            interp.set_error(Error::from(err));
                            Ok(())
                        }
                    }
                } else {
                    Ok(()) // TODO: ACV?
                }
            }
        }
    };
}

use lc3_traits::peripherals::gpio::{Gpio, GpioPin::*, GpioPin, GpioBank};

gpio_mem_mapped!(G0, 'A', "G0", GA0CR, GA0DR, GA0_CR_ADDR, GA0_DR_ADDR, GA0_INT_VEC);
gpio_mem_mapped!(G1, 'A', "G1", GA1CR, GA1DR, GA1_CR_ADDR, GA1_DR_ADDR, GA1_INT_VEC);
gpio_mem_mapped!(G2, 'A', "G2", GA2CR, GA2DR, GA2_CR_ADDR, GA2_DR_ADDR, GA2_INT_VEC);
gpio_mem_mapped!(G3, 'A', "G3", GA3CR, GA3DR, GA3_CR_ADDR, GA3_DR_ADDR, GA3_INT_VEC);
gpio_mem_mapped!(G4, 'A', "G4", GA4CR, GA4DR, GA4_CR_ADDR, GA4_DR_ADDR, GA4_INT_VEC);
gpio_mem_mapped!(G5, 'A', "G5", GA5CR, GA5DR, GA5_CR_ADDR, GA5_DR_ADDR, GA5_INT_VEC);
gpio_mem_mapped!(G6, 'A', "G6", GA6CR, GA6DR, GA6_CR_ADDR, GA6_DR_ADDR, GA6_INT_VEC);
gpio_mem_mapped!(G7, 'A', "G7", GA7CR, GA7DR, GA7_CR_ADDR, GA7_DR_ADDR, GA7_INT_VEC);

gpio_mem_mapped!(G0, 'B', "G0", GB0CR, GB0DR, GB0_CR_ADDR, GB0_DR_ADDR, GB0_INT_VEC);
gpio_mem_mapped!(G1, 'B', "G1", GB1CR, GB1DR, GB1_CR_ADDR, GB1_DR_ADDR, GB1_INT_VEC);
gpio_mem_mapped!(G2, 'B', "G2", GB2CR, GB2DR, GB2_CR_ADDR, GB2_DR_ADDR, GB2_INT_VEC);
gpio_mem_mapped!(G3, 'B', "G3", GB3CR, GB3DR, GB3_CR_ADDR, GB3_DR_ADDR, GB3_INT_VEC);
gpio_mem_mapped!(G4, 'B', "G4", GB4CR, GB4DR, GB4_CR_ADDR, GB4_DR_ADDR, GB4_INT_VEC);
gpio_mem_mapped!(G5, 'B', "G5", GB5CR, GB5DR, GB5_CR_ADDR, GB5_DR_ADDR, GB5_INT_VEC);
gpio_mem_mapped!(G6, 'B', "G6", GB6CR, GB6DR, GB6_CR_ADDR, GB6_DR_ADDR, GB6_INT_VEC);
gpio_mem_mapped!(G7, 'B', "G7", GB7CR, GB7DR, GB7_CR_ADDR, GB7_DR_ADDR, GB7_INT_VEC);

gpio_mem_mapped!(G0, 'C', "G0", GC0CR, GC0DR, GC0_CR_ADDR, GC0_DR_ADDR, GC0_INT_VEC);
gpio_mem_mapped!(G1, 'C', "G1", GC1CR, GC1DR, GC1_CR_ADDR, GC1_DR_ADDR, GC1_INT_VEC);
gpio_mem_mapped!(G2, 'C', "G2", GC2CR, GC2DR, GC2_CR_ADDR, GC2_DR_ADDR, GC2_INT_VEC);
gpio_mem_mapped!(G3, 'C', "G3", GC3CR, GC3DR, GC3_CR_ADDR, GC3_DR_ADDR, GC3_INT_VEC);
gpio_mem_mapped!(G4, 'C', "G4", GC4CR, GC4DR, GC4_CR_ADDR, GC4_DR_ADDR, GC4_INT_VEC);
gpio_mem_mapped!(G5, 'C', "G5", GC5CR, GC5DR, GC5_CR_ADDR, GC5_DR_ADDR, GC5_INT_VEC);
gpio_mem_mapped!(G6, 'C', "G6", GC6CR, GC6DR, GC6_CR_ADDR, GC6_DR_ADDR, GC6_INT_VEC);
gpio_mem_mapped!(G7, 'C', "G7", GC7CR, GC7DR, GC7_CR_ADDR, GC7_DR_ADDR, GC7_INT_VEC);


// TODO!
/* pub struct GPIOA_DR(Word);
impl Deref for GPIOA_DR {

    type Target = Word;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl MemMapped for GPIOA_DR {
    const ADDR: Addr = GPIOA_DR_ADDR;

    fn with_value(value: Word) -> Self { Self(value) }

    fn from<I: Ipa> (interp: &I) -> Result<Self, Acv> {
        let readings = Gpio::read_all(interp.get_gpio());

        let mut word: Word = readings
            .iter()
            .enumerate()
            .filter_map(|(idx, r)| r.map(|b| (idx, b as Word)).ok())
            .fold(0, |acc, (idx, r)| acc | (r << idx as Word));

        if readings.iter().any(|r| r.is_err()) {
            word = word | 0x8000;
        }

        Ok(Self::with_value(word))
    }

    fn set<I: Ipa> (interp: &mut I, value: Word) -> WriteAttempt {
        let mut values = GpioPinArr([false; GpioPin::NUM_PINS]);

        GPIO_PINS
            .iter()
            .enumerate()
            .for_each(|(idx, pin)| {
                values[*pin] = value.bit(idx as u32);
            });

        Gpio::write_all(interp.get_gpio_mut(), values);

        Ok(())
    }
} */

// Idk how to coerce the state of all pins into a word
//#[doc="GPIO Control Register, all pins"]
//#[derive(Copy, Clone, Debug, PartialEq)]
//pub struct GPIOCR(Word);
//impl Deref for GPIOCR {
//    type Target = Word;
//    fn deref(&self) -> &Self::Target { &self.0 }
//}
//impl MemMapped for GPIOCR {
//    const ADDR: Addr = 0xFE17;
//
//    fn with_value(valWord: Ipa) -> Self { Self(value)  /    }
//}

macro_rules! adc_mem_mapped {
    ($pin:expr, $pin_name:literal, $cr:ident, $dr:ident, $cr_addr:expr, $dr_addr:expr) => {
        #[doc=$pin_name]
        #[doc="ADC Pin Control Register"] // TODO: format correctly
        #[derive(Copy, Clone, Debug, PartialEq)]
        pub struct $cr(Word);

        impl Deref for $cr {
            type Target = Word;

            fn deref(&self) -> &Self::Target { &self.0 }
        }

        impl MemMapped for $cr {
            const ADDR: Addr = $cr_addr;

            fn with_value(value: Word) -> Self { Self(value) }

            fn from<I: Ipa> (interp: &I) -> Result<Self, Acv> {
                let state = Adc::get_state(interp.get_adc(), $pin);

                use lc3_traits::peripherals::adc::AdcState::*;
                let word: Word = match state {
                    Disabled => 0,
                    Enabled => 1,
                };

                Ok(Self::with_value(word))
            }

            fn set<I: Ipa>(interp: &mut I, value: Word) -> WriteAttempt {
                use lc3_traits::peripherals::adc::AdcState::*;
                let state = match value.bit(0) {
                    false => Disabled,
                    true => Enabled,
                };

                match Adc::set_state(interp.get_adc_mut(), $pin, state) {
                    Ok(()) => Ok(()),
                    Err(err) => {
                        interp.set_error(Error::from(err));
                        Ok(())
                    }
                }
            }
        }

        #[doc=$pin_name]
        #[doc="ADC Pin Data Register"] // TODO: format correctly
        #[derive(Copy, Clone, Debug, PartialEq)]
        pub struct $dr(Word);

        impl Deref for $dr {
            type Target = Word;

            fn deref(&self) -> &Self::Target { &self.0 }
        }

        impl MemMapped for $dr {
            const ADDR: Addr = $dr_addr;

            fn with_value(value: Word) -> Self { Self(value) }

            fn from<I: Ipa> (interp: &I) -> Result<Self, Acv> {

                let word = match Adc::read(interp.get_adc(), $pin) {
                    Ok(val) => val as Word,
                    Err(err) => {
                        interp.set_error(Error::from(err));
                        0x8000
                    }
                };

                Ok(Self::with_value(word))
            }

            fn set<I: Ipa>(_interp: &mut I, _value: Word) -> WriteAttempt {
                Ok(())      // TODO: Ignore writes to ADC data register?
            }
        }
    };
}

use lc3_traits::peripherals::adc::{Adc, AdcPin::*};

adc_mem_mapped!(A0, "A0", A0CR, A0DR, A0CR_ADDR, A0DR_ADDR);
adc_mem_mapped!(A1, "A1", A1CR, A1DR, A1CR_ADDR, A1DR_ADDR);
adc_mem_mapped!(A2, "A2", A2CR, A2DR, A2CR_ADDR, A2DR_ADDR);
adc_mem_mapped!(A3, "A3", A3CR, A3DR, A3CR_ADDR, A3DR_ADDR);
adc_mem_mapped!(A4, "A4", A4CR, A4DR, A4CR_ADDR, A4DR_ADDR);
adc_mem_mapped!(A5, "A5", A5CR, A5DR, A5CR_ADDR, A5DR_ADDR);

use lc3_traits::peripherals::clock::Clock;
#[doc = "Clock Register"]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CLKR(Word);
impl Deref for CLKR {
    type Target = Word;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl MemMapped for CLKR {
    const ADDR: Addr = CLKR_ADDR;

    fn with_value(value: Word) -> Self {
        Self(value)
    }

    fn from<I: Ipa>(interp: &I) -> Result<Self, Acv> {
        Ok(Self::with_value(Clock::get_milliseconds(
            interp.get_clock(),
        )))
    }

    fn set<I: Ipa>(interp: &mut I, value: Word) -> WriteAttempt {
        Clock::set_milliseconds(interp.get_clock_mut(), value);

        Ok(())
    }
}

macro_rules! pwm_mem_mapped {
    ($pin:expr, $pin_name:literal, $cr:ident, $dr:ident, $cr_addr:expr, $dr_addr:expr) => {
        #[doc=$pin_name]
        #[doc="PWM Pin Control Register"] // TODO: format correctly
        #[derive(Copy, Clone, Debug, PartialEq)]
        pub struct $cr(Word);

        impl Deref for $cr {
            type Target = Word;

            fn deref(&self) -> &Self::Target { &self.0 }
        }

        impl MemMapped for $cr {
            const ADDR: Addr = $cr_addr;

            fn with_value(value: Word) -> Self { Self(value) }

            fn from<I: Ipa> (interp: &I) -> Result<Self, Acv> {
                let state = Pwm::get_state(interp.get_pwm(), $pin);

                use lc3_traits::peripherals::pwm::PwmState::*;
                let word: Word = match state {
                    Disabled => 0,
                    Enabled(ref nzu8) => nzu8.get() as Word,
                };

                Ok(Self::with_value(word))
            }

            fn set<I: Ipa>(interp: &mut I, value: Word) -> WriteAttempt {
                use lc3_traits::peripherals::pwm::PwmState::*;
                use core::num::NonZeroU8;

                let state_val: u8 = value as u8;
                let state = match state_val {
                    0 => Disabled,
                    _ => Enabled(NonZeroU8::new(state_val).unwrap()),
                };

                Pwm::set_state(interp.get_pwm_mut(), $pin, state);

                Ok(())
            }
        }

        #[doc=$pin_name]
        #[doc="PWM Pin Duty Cycle Register"] // TODO: format correctly
        #[derive(Copy, Clone, Debug, PartialEq)]
        pub struct $dr(Word);

        impl Deref for $dr {
            type Target = Word;

            fn deref(&self) -> &Self::Target { &self.0 }
        }

        impl MemMapped for $dr {
            const ADDR: Addr = $dr_addr;

            fn with_value(value: Word) -> Self { Self(value) }

            fn from<I: Ipa> (interp: &I) -> Result<Self, Acv> { // TODO: change all these to some other kind of error since we already check for ACVs in read_word, etc. {
                let word = Pwm::get_duty_cycle(interp.get_pwm(), $pin) as Word;

                Ok(Self::with_value(word))
            }

            fn set<I: Ipa>(interp: &mut I, value: Word) -> WriteAttempt {
                let duty_val: u8 = value as u8;

                Pwm::set_duty_cycle(interp.get_pwm_mut(), $pin, duty_val);

                Ok(())
            }
        }
    };
}

use lc3_traits::peripherals::pwm::{Pwm, PwmPin::*};

pwm_mem_mapped!(P0, "P0", P0CR, P0DR, P0CR_ADDR, P0DR_ADDR);
pwm_mem_mapped!(P1, "P1", P1CR, P1DR, P1CR_ADDR, P1DR_ADDR);

macro_rules! timer_mem_mapped {
    ($id:expr, $id_name:literal, $cr:ident, $dr:ident, $cr_addr:expr, $dr_addr:expr, $int_vec:expr) => {
        #[doc=$id_name]
        #[doc="Timer Control Register"] // TODO: format correctly
        #[derive(Copy, Clone, Debug, PartialEq)]
        pub struct $cr(Word);

        impl Deref for $cr {
            type Target = Word;

            fn deref(&self) -> &Self::Target { &self.0 }
        }

        impl MemMapped for $cr {
            const ADDR: Addr = $cr_addr;

            fn with_value(value: Word) -> Self { Self(value) }

            fn from<I: Ipa> (interp: &I) -> Result<Self, Acv> {
                let mode = Timers::get_mode(interp.get_timers(), $id);

                use lc3_traits::peripherals::timers::TimerMode::*;
                let word: Word = match mode {
                    SingleShot => 0,
                    Repeated => 1,
                };

                Ok(Self::with_value(word))
            }

            fn set<I: Ipa>(interp: &mut I, value: Word) -> WriteAttempt {
                use lc3_traits::peripherals::timers::TimerMode::*;
                let mode = if value.bit(0) {
                    Repeated
                } else {
                    SingleShot
                };

                Timers::set_mode(interp.get_timers_mut(), $id, mode);

                Ok(())
            }
        }

        impl Interrupt for $cr {
            const INT_VEC: u8 = $int_vec;
            const PRIORITY: u8 = TIMER_INT_PRIORITY;

            fn interrupt_ready<I: Ipa>(interp: &I) -> bool {
                Timers::interrupt_occurred(interp.get_timers(), $id)
            }

            fn interrupt_enabled<I: Ipa>(interp: &I) -> bool {
                Timers::interrupts_enabled(interp.get_timers(), $id)
            }

            fn reset_interrupt_flag<I: Ipa>(interp: &mut I) {
                if Timers::interrupts_enabled(interp.get_timers(), $id) {
                    Timers::reset_interrupt_flag(interp.get_timers_mut(), $id);
                }
            }

        }

        #[doc=$id_name]
        #[doc="Timer Period Register"] // TODO: format correctly
        #[derive(Copy, Clone, Debug, PartialEq)]
        pub struct $dr(Word);

        impl Deref for $dr {
            type Target = Word;

            fn deref(&self) -> &Self::Target { &self.0 }
        }

        impl MemMapped for $dr {
            const ADDR: Addr = $dr_addr;

            fn with_value(value: Word) -> Self { Self(value) }

            // TODO: change all these to some other kind of error since we already check for ACVs in read_word, etc.
            fn from<I: Ipa> (interp: &I) -> Result<Self, Acv> {
                let state = Timers::get_state(interp.get_timers(), $id);

                use lc3_traits::peripherals::timers::TimerState::*;
                let value = match state {
                    Disabled => 0,
                    WithPeriod(period) => period.into(),
                };

                Ok(Self::with_value(value))
            }

            fn set<I: Ipa>(interp: &mut I, value: Word) -> WriteAttempt {
                use lc3_traits::peripherals::timers::TimerState::*;
                use lc3_traits::peripherals::timers::Period;
                let state = match value {
                    0 => Disabled,
                    nonzero => WithPeriod(Period::new(nonzero).unwrap()), // TODO: will this fail?
                };
                Timers::set_state(interp.get_timers_mut(), $id, state);

                Ok(())
            }
        }
    };
}

use lc3_traits::peripherals::timers::{Timers, TimerId::*};

timer_mem_mapped!(T0, "T0", T0CR, T0DR, T0CR_ADDR, T0DR_ADDR, T0_INT_VEC);
timer_mem_mapped!(T1, "T1", T1CR, T1DR, T1CR_ADDR, T1DR_ADDR, T1_INT_VEC);

mem_mapped!(special: BSP, BSP_ADDR, "Backup Stack Pointer.");

mem_mapped!(special: PSR, PSR_ADDR, "Program Status Register.");

use lc3_traits::control::ProcessorMode;

impl PSR {
    pub fn get_priority(&self) -> u8 {
        self.u8(8..10)
    }

    pub fn set_priority<I: Ipa>(&mut self, interp: &mut I, priority: u8) {
        self.0 = (self.0 & (!WORD_MAX_VAL.select(8..10))) | ((priority as Word).u16(0..2) << 8);

        // Don't return a `WriteAttempt` since PSR accesses don't produce ACVs (and are hence infallible).
        self.write_current_value(interp).unwrap();
    }

    pub fn get_mode(&self) -> ProcessorMode {
        if self.in_user_mode() {
            ProcessorMode::User
        } else {
            ProcessorMode::Supervisor
        }
    }

    pub fn in_user_mode(&self) -> bool {
        self.bit(15)
    }
    pub fn in_privileged_mode(&self) -> bool {
        !self.in_user_mode()
    }

    fn set_mode<I: Ipa>(&mut self, interp: &mut I, processor_mode: ProcessorMode) {
        self.0 = self.0.u16(0..14) | (Into::<Word>::into(processor_mode) << 15);

        // Don't return a `WriteAttempt` since PSR accesses are infallible.
        self.write_current_value(interp).unwrap()
    }

    pub fn to_user_mode<I: Ipa>(&mut self, interp: &mut I) {
        self.set_mode(interp, ProcessorMode::User)
    }

    pub fn to_privileged_mode<I: Ipa>(&mut self, interp: &mut I) {
        self.set_mode(interp, ProcessorMode::Supervisor)
    }

    pub fn n(&self) -> bool {
        self.bit(2)
    }
    pub fn z(&self) -> bool {
        self.bit(1)
    }
    pub fn p(&self) -> bool {
        self.bit(0)
    }
    pub fn get_cc(&self) -> (bool, bool, bool) {
        (self.n(), self.z(), self.p())
    }

    pub fn set_cc<I: Ipa>(&mut self, interp: &mut I, word: Word) {
        let word = word as SignedWord;

        // checking for n is easy once we've got a `SignedWord`.
        let n: bool = word.is_negative();

        // z is easy enough to check for:
        let z: bool = word == 0;

        // if we're not negative or zero, we're positive:
        let p: bool = !(n | z);

        fn bit_to_word(bit: bool, left_shift: u32) -> u16 {
            (if bit { 1 } else { 0 }) << left_shift
        }

        let b = bit_to_word;

        self.0 = (self.0 & !(WORD_MAX_VAL.select(0..2))) | b(n, 2) | b(z, 1) | b(p, 0);

        // Don't return a `WriteAttempt` since PSR accesses are infallible.
        self.write_current_value(interp).unwrap();
    }
}

// mem_mapped!(special: MCR, MCR_ADDRESS, "Machine Control Register.");

/// Machine Control Register
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MCR(Word);

impl Deref for MCR {
    type Target = Word;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MemMapped for MCR {
    const ADDR: Addr = MCR_ADDR;
    fn with_value(value: Word) -> Self {
        Self(value)
    }
    fn from<I: Ipa>(interp: &I) -> Result<Self, Acv> {
        Ok(Self::with_value(
            interp.get_word_force_memory_backed(Self::ADDR),
        ))
    }

    fn set<I: Ipa>(
        interp: &mut I,
        value: Word,
    ) -> WriteAttempt {
        interp.set_word_force_memory_backed(Self::ADDR, value);

        if !value.bit(15) {
            interp.halt();
        }

        Ok(())
    }
}

impl MemMappedSpecial for MCR {}

impl MCR {
    fn set_running_bit<I: Ipa>(&mut self, interp: &mut I, bit: bool) {
        self.0 = (self.0 & (!WORD_MAX_VAL.select(15..15))) | ((bit as Word) << 15);

        // Don't return a `WriteAttempt` since MCR accesses don't produce ACVs (and are hence infallible).
        self.write_current_value(interp).unwrap();
    }

    pub fn is_running(&self) -> bool {
        self.0.bit(15)
    }

    pub fn halt<I: Ipa>(&mut self, interp: &mut I) {
        self.set_running_bit(interp, false);
    }

    pub fn run<I: Ipa>(&mut self, interp: &mut I) {
        self.set_running_bit(interp, true);
    }
}

// TODO: error set memory mapped device! (for TRAPs to set to communicate errors
// to the tui/Control)
