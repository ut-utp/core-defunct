//! TODO!

use lc3_isa::{MEM_MAPPED_START_ADDR, INTERRUPT_SERVICE_ROUTINES_START_ADDR};

// TODO: split into modules/structs
pub const KBSR_ADDR: Addr = 0xFE00;
pub const KBDR_ADDR: Addr = 0xFE02;

pub const KEYBOARD_INT_VEC: u8 = 0x80;
pub const KEYBOARD_INT_PRIORITY: u8 = 4;

pub const DSR_ADDR: Addr = 0xFE04;
pub const DDR_ADDR: Addr = 0xFE06;

pub const DISPLAY_INT_VEC: u8 = 0x81; // TODO: What is this actually?
pub const DISPLAY_INT_PRIORITY: u8 = 4;

pub const GPIO_OFFSET: u8 = 0x30;
const GPIO_MEM_MAPPED_BASE: Addr = MEM_MAPPED_START_ADDR + (GPIO_OFFSET as Addr);
const GPIO_PIN_ADDRS: Addr = 2;

pub const G0CR_ADDR: Addr = gpio_cr_addr(0); // xFE30
pub const G0DR_ADDR: Addr = gpio_dr_addr(0); // xFE31
pub const G1CR_ADDR: Addr = gpio_cr_addr(1); // xFE32
pub const G1DR_ADDR: Addr = gpio_dr_addr(1); // xFE33
pub const G2CR_ADDR: Addr = gpio_cr_addr(2); // xFE34
pub const G2DR_ADDR: Addr = gpio_dr_addr(2); // xFE35
pub const G3CR_ADDR: Addr = gpio_cr_addr(3); // xFE36
pub const G3DR_ADDR: Addr = gpio_dr_addr(3); // xFE37
pub const G4CR_ADDR: Addr = gpio_cr_addr(4); // xFE38
pub const G4DR_ADDR: Addr = gpio_dr_addr(4); // xFE39
pub const G5CR_ADDR: Addr = gpio_cr_addr(5); // xFE3A
pub const G5DR_ADDR: Addr = gpio_dr_addr(5); // xFE3B
pub const G6CR_ADDR: Addr = gpio_cr_addr(6); // xFE3C
pub const G6DR_ADDR: Addr = gpio_dr_addr(6); // xFE3D
pub const G7CR_ADDR: Addr = gpio_cr_addr(7); // xFE3E
pub const G7DR_ADDR: Addr = gpio_dr_addr(7); // xFE3F

const fn gpio_cr_addr(i: u16) -> Addr {
    GPIO_MEM_MAPPED_BASE + GPIO_PIN_ADDRS * i
}
const fn gpio_dr_addr(i: u16) -> Addr {
    gpio_cr_addr(i) + 1
}

pub const GPIODR_ADDR: Addr = GPIO_MEM_MAPPED_BASE + GPIO_PIN_ADDRS * 8 + 0;

pub const GPIO_BASE_INT_VEC: Addr = INTERRUPT_SERVICE_ROUTINES_START_ADDR + (GPIO_OFFSET as Addr); // x1B0
pub const G0_INT_VEC: u8 = 128 + GPIO_OFFSET + 0; // xB0
pub const G1_INT_VEC: u8 = 128 + GPIO_OFFSET + 1; // xB1
pub const G2_INT_VEC: u8 = 128 + GPIO_OFFSET + 2; // xB2
pub const G3_INT_VEC: u8 = 128 + GPIO_OFFSET + 3; // xB3
pub const G4_INT_VEC: u8 = 128 + GPIO_OFFSET + 4; // xB4
pub const G5_INT_VEC: u8 = 128 + GPIO_OFFSET + 5; // xB5
pub const G6_INT_VEC: u8 = 128 + GPIO_OFFSET + 6; // xB6
pub const G7_INT_VEC: u8 = 128 + GPIO_OFFSET + 7; // xB7
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

pub const BSP_ADDR: Addr = 0xFFFA;

use crate::interp::InstructionInterpreterPeripheralAccess;
use core::ops::Deref;
use lc3_isa::{Addr, Bits, SignedWord, Word, MCR as MCR_ADDR, PSR as PSR_ADDR, WORD_MAX_VAL};
use lc3_traits::peripherals::Peripherals;
use lc3_traits::error::Error;

use crate::interp::{Acv, WriteAttempt};

pub trait MemMapped: Deref<Target = Word> + Sized {
    const ADDR: Addr;
    const HAS_STATEFUL_READS: bool = false;

    fn with_value(value: Word) -> Self;

    fn from<'a, I>(interp: &I) -> Result<Self, Acv>
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
        // Checked access by default:
        Ok(Self::with_value(interp.get_word(Self::ADDR)?))
    }

    fn set<'a, I>(interp: &mut I, value: Word) -> WriteAttempt
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
        // Checked access by default:
        interp.set_word(Self::ADDR, value)
    }

    fn update<'a, I>(interp: &mut I, func: impl FnOnce(Self) -> Word) -> WriteAttempt
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
        Self::set(interp, func(Self::from(interp)?))
    }

    #[doc(hidden)]
    fn write_current_value<'a, I>(&self, interp: &mut I) -> WriteAttempt
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
        Self::set(interp, **self)
    }
}

// Don't implement this manually; you could mess up. (only implement this if
// you've overriden the default impls for from and set in the trait above).
//
// Use the macro below instead.
pub trait MemMappedSpecial: MemMapped {
    // Infallible.
    fn from_special<'a, I>(interp: &I) -> Self
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
        Self::from(interp).unwrap()
    }

    // Also infallible.
    fn set_special<'a, I>(interp: &mut I, value: Word)
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
        Self::set(interp, value).unwrap()
    }
}

pub trait Interrupt: MemMapped {
    const INT_VEC: u8;
    const PRIORITY: u8; // Must be a 3 bit number

    /// Returns true if:
    ///   - this particular interrupt is enabled
    ///   - this particular interrupt is ready to fire (i.e. pending).
    fn interrupt<'a, I>(interp: &I) -> bool
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
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
    fn interrupt_ready<'a, I>(interp: &I) -> bool
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>;

    /// Returns true if the interrupt is enabled.
    fn interrupt_enabled<'a, I>(interp: &I) -> bool
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>;

    fn reset_interrupt_flag<'a, I>(interp: &mut I)
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>;
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
        fn from<'a, I: InstructionInterpreterPeripheralAccess<'a>>(interp: &I) -> Result<Self, Acv>
        where
            <I as Deref>::Target: Peripherals<'a>,
        {
            // Special unchecked access!
            Ok(Self::with_value(
                interp.get_word_force_memory_backed(Self::ADDR),
            ))
        }

        fn set<'a, I: InstructionInterpreterPeripheralAccess<'a>>(
            interp: &mut I,
            value: Word,
        ) -> WriteAttempt
        where
            <I as Deref>::Target: Peripherals<'a>,
        {
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

    fn from<'a, I>(interp: &I) -> Result<Self, Acv>
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
        let data = match Input::read_data(interp.get_peripherals()) {
            Ok(val) => val as Word,
            Err(err) => {
                interp.set_error(Error::from(err));
                0
            }
        };

        Ok(Self::with_value(data))
    }

    fn set<'a, I>(_interp: &mut I, _value: Word) -> WriteAttempt
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
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

    fn from<'a, I>(interp: &I) -> Result<Self, Acv>
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
        // Bit 15: Ready
        // Bit 14: Interrupt Enabled
        let word = ((Input::current_data_unread(interp.get_peripherals()) as Word) << 15)
            | ((Input::interrupts_enabled(interp.get_peripherals()) as Word) << 14);

        Ok(Self::with_value(word))
    }

    fn set<'a, I>(interp: &mut I, value: Word) -> WriteAttempt
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
        // Bit 15: Ready
        // Bit 14: Interrupt Enabled
        let interrupt_enabled_bit = value.bit(14);

        Input::set_interrupt_enable_bit(interp.get_peripherals_mut(), interrupt_enabled_bit); // TODO: do something on error

        Ok(())
    }
}

impl Interrupt for KBSR {
    const INT_VEC: u8 = KEYBOARD_INT_VEC;
    const PRIORITY: u8 = KEYBOARD_INT_PRIORITY;

    fn interrupt_ready<'a, I>(interp: &I) -> bool
        where
            I: InstructionInterpreterPeripheralAccess<'a>,
            <I as Deref>::Target: Peripherals<'a>,
    {
        Input::interrupt_occurred(interp.get_peripherals())
    }

    fn interrupt_enabled<'a, I>(interp: &I) -> bool
        where
            I: InstructionInterpreterPeripheralAccess<'a>,
            <I as Deref>::Target: Peripherals<'a>
    {
        Input::interrupts_enabled(interp.get_peripherals())
    }

    fn reset_interrupt_flag<'a, I>(interp: &mut I)
        where
            I: InstructionInterpreterPeripheralAccess<'a>,
            <I as Deref>::Target: Peripherals<'a>
    {
        if Input::interrupts_enabled(interp.get_peripherals()) {
            Input::reset_interrupt_flag(interp.get_peripherals_mut());
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

    fn from<'a, I>(interp: &I) -> Result<Self, Acv>
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
        Ok(Self::with_value(
            (Output::current_data_written(interp.get_peripherals()) as Word) << 15,
        ))
    }

    fn set<'a, I>(interp: &mut I, value: Word) -> WriteAttempt
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
        Output::set_interrupt_enable_bit(interp.get_peripherals_mut(), value.bit(1));
        Ok(())
    }
}

impl Interrupt for DSR {
    const INT_VEC: u8 = DISPLAY_INT_VEC;
    const PRIORITY: u8 = DISPLAY_INT_PRIORITY;

    fn interrupt_ready<'a, I>(interp: &I) -> bool
        where
            I: InstructionInterpreterPeripheralAccess<'a>,
            <I as Deref>::Target: Peripherals<'a>,
    {
        Output::interrupt_occurred(interp.get_peripherals())
    }

    fn interrupt_enabled<'a, I>(interp: &I) -> bool
        where
            I: InstructionInterpreterPeripheralAccess<'a>,
            <I as Deref>::Target: Peripherals<'a>
    {
        Output::interrupts_enabled(interp.get_peripherals())
    }

    fn reset_interrupt_flag<'a, I>(interp: &mut I)
        where
            I: InstructionInterpreterPeripheralAccess<'a>,
            <I as Deref>::Target: Peripherals<'a>
    {
        if Output::interrupts_enabled(interp.get_peripherals()) {
            Output::reset_interrupt_flag(interp.get_peripherals_mut());
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

    fn from<'a, I>(_interp: &I) -> Result<Self, Acv>
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
        // TODO: error here?
        Ok(Self::with_value(0 as Word))
    }

    fn set<'a, I>(interp: &mut I, value: Word) -> WriteAttempt
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
        // TODO: propagate errors here!
        let _ = Output::write_data(interp.get_peripherals_mut(), value as u8);
        Ok(())
    }
}

macro_rules! gpio_mem_mapped {
    ($pin:expr, $pin_name:literal, $cr:ident, $dr:ident, $cr_addr:expr, $dr_addr:expr, $int_vec:expr) => {
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

            fn from<'a, I> (interp: &I) -> Result<Self, Acv>
            where
                I: InstructionInterpreterPeripheralAccess<'a>,
                <I as Deref>::Target: Peripherals<'a>,
            {
                let state = Gpio::get_state(interp.get_peripherals(), $pin);

                use lc3_traits::peripherals::gpio::GpioState::*;
                let word: Word = match state {
                    Disabled => 0,
                    Output => 1,
                    Input => 2,
                    Interrupt => 3,
                };

                Ok(Self::with_value(word))
            }

            fn set<'a, I>(interp: &mut I, value: Word) -> WriteAttempt
            where
                I: InstructionInterpreterPeripheralAccess<'a>,
                <I as Deref>::Target: Peripherals<'a>,
            {
                use lc3_traits::peripherals::gpio::GpioState::*;
                let state = match value.bits(0..1) {
                    0 => Disabled,
                    1 => Output,
                    2 => Input,
                    3 => Interrupt,
                    _ => unreachable!()
                };

                match Gpio::set_state(interp.get_peripherals_mut(), $pin, state) {
                    Ok(()) => Ok(()),
                    Err(err) => {
                        interp.set_error(Error::from(err));
                        Ok(())
                    }
                }
            }
        }

        impl Interrupt for $cr {
            const INT_VEC: u8 = $int_vec;
            const PRIORITY: u8 = GPIO_INT_PRIORITY;

            fn interrupt_ready<'a, I>(interp: &I) -> bool
            where
                I: InstructionInterpreterPeripheralAccess<'a>,
                <I as Deref>::Target: Peripherals<'a>,
            {
                Gpio::interrupt_occurred(interp.get_peripherals(), $pin)
            }

            fn interrupt_enabled<'a, I>(interp: &I) -> bool
            where
                I: InstructionInterpreterPeripheralAccess<'a>,
                <I as Deref>::Target: Peripherals<'a>
            {
                Gpio::interrupts_enabled(interp.get_peripherals(), $pin)
            }

            fn reset_interrupt_flag<'a, I>(interp: &mut I)
                where
                    I: InstructionInterpreterPeripheralAccess<'a>,
                    <I as Deref>::Target: Peripherals<'a>
            {
                if Gpio::interrupts_enabled(interp.get_peripherals(), $pin) {
                    Gpio::reset_interrupt_flag(interp.get_peripherals_mut(), $pin);
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

            fn from<'a, I>(interp: &I) -> Result<Self, Acv> // TODO: change all these to some other kind of error since we already check for ACVs in read_word, etc.
            where
                I: InstructionInterpreterPeripheralAccess<'a>,
                <I as Deref>::Target: Peripherals<'a>,
            {
                let word = match Gpio::read(interp.get_peripherals(), $pin) {
                    Ok(val) => val as Word,
                    Err(err) => {
                        interp.set_error(Error::from(err));
                        0x8000
                    }
                };

                Ok(Self::with_value(word))
            }

            fn set<'a, I>(interp: &mut I, value: Word) -> WriteAttempt
            where
                I: InstructionInterpreterPeripheralAccess<'a>,
                <I as Deref>::Target: Peripherals<'a>,
            {
                let bit: bool = value.bit(0);
                match Gpio::write(interp.get_peripherals_mut(), $pin, bit) {
                    Ok(()) => Ok(()),
                    Err(err) => {
                        interp.set_error(Error::from(err));
                        Ok(())
                    }
                }
            }
        }
    };
}

use lc3_traits::peripherals::gpio::{Gpio, GpioPin::*, GpioPinArr, GpioPin, GPIO_PINS};

gpio_mem_mapped!(G0, "G0", G0CR, G0DR, G0CR_ADDR, G0DR_ADDR, G0_INT_VEC);
gpio_mem_mapped!(G1, "G1", G1CR, G1DR, G1CR_ADDR, G1DR_ADDR, G1_INT_VEC);
gpio_mem_mapped!(G2, "G2", G2CR, G2DR, G2CR_ADDR, G2DR_ADDR, G2_INT_VEC);
gpio_mem_mapped!(G3, "G3", G3CR, G3DR, G3CR_ADDR, G3DR_ADDR, G3_INT_VEC);
gpio_mem_mapped!(G4, "G4", G4CR, G4DR, G4CR_ADDR, G4DR_ADDR, G4_INT_VEC);
gpio_mem_mapped!(G5, "G5", G5CR, G5DR, G5CR_ADDR, G5DR_ADDR, G5_INT_VEC);
gpio_mem_mapped!(G6, "G6", G6CR, G6DR, G6CR_ADDR, G6DR_ADDR, G6_INT_VEC);
gpio_mem_mapped!(G7, "G7", G7CR, G7DR, G7CR_ADDR, G7DR_ADDR, G7_INT_VEC);

pub struct GPIODR(Word);
impl Deref for GPIODR {

    type Target = Word;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl MemMapped for GPIODR {
    const ADDR: Addr = GPIODR_ADDR;

    fn with_value(value: Word) -> Self { Self(value) }

    fn from<'a, I> (interp: &I) -> Result<Self, Acv>
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
        let readings = Gpio::read_all(interp.get_peripherals());

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

    fn set<'a, I> (interp: &mut I, value: Word) -> WriteAttempt
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
        let mut values = GpioPinArr([false; GpioPin::NUM_PINS]);

        GPIO_PINS
            .iter()
            .enumerate()
            .for_each(|(idx, pin)| {
                values[*pin] = value.bit(idx as u32);
            });

        Gpio::write_all(interp.get_peripherals_mut(), values);

        Ok(())
    }
}

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
//    fn with_value(value: Word) -> Self { Self(value) }
//
//    fn from<I: InstructionInterpreterPeripheralAccess> (interp: &I) -> Result<Self, Acv>
//    where for <'a> <I as Deref>::Target: Peripherals<'a> {
//
//    }
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

            fn from<'a, I> (interp: &I) -> Result<Self, Acv>
            where
                I: InstructionInterpreterPeripheralAccess<'a>,
                <I as Deref>::Target: Peripherals<'a>,
            {
                let state = Adc::get_state(interp.get_peripherals(), $pin);

                use lc3_traits::peripherals::adc::AdcState::*;
                let word: Word = match state {
                    Disabled => 0,
                    Enabled => 1,
                };

                Ok(Self::with_value(word))
            }

            fn set<'a, I>(interp: &mut I, value: Word) -> WriteAttempt
            where
                I: InstructionInterpreterPeripheralAccess<'a>,
                <I as Deref>::Target: Peripherals<'a>,
            {
                use lc3_traits::peripherals::adc::AdcState::*;
                let state = match value.bit(0) {
                    false => Disabled,
                    true => Enabled,
                };

                match Adc::set_state(interp.get_peripherals_mut(), $pin, state) {
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

            fn from<'a, I> (interp: &I) -> Result<Self, Acv>
            where
                I: InstructionInterpreterPeripheralAccess<'a>,
                <I as Deref>::Target: Peripherals<'a>,
            {

                let word = match Adc::read(interp.get_peripherals(), $pin) {
                    Ok(val) => val as Word,
                    Err(err) => {
                        interp.set_error(Error::from(err));
                        0x8000
                    }
                };

                Ok(Self::with_value(word))
            }

            fn set<'a, I>(_interp: &mut I, _value: Word) -> WriteAttempt
            where
                I: InstructionInterpreterPeripheralAccess<'a>,
                <I as Deref>::Target: Peripherals<'a>,
            {
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

    fn from<'a, I>(interp: &I) -> Result<Self, Acv>
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
        Ok(Self::with_value(Clock::get_milliseconds(
            interp.get_peripherals(),
        )))
    }

    fn set<'a, I>(interp: &mut I, value: Word) -> WriteAttempt
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
        Clock::set_milliseconds(interp.get_peripherals_mut(), value);

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

            fn from<'a, I> (interp: &I) -> Result<Self, Acv>
            where
                I: InstructionInterpreterPeripheralAccess<'a>,
                <I as Deref>::Target: Peripherals<'a>,
            {
                let state = Pwm::get_state(interp.get_peripherals(), $pin);

                use lc3_traits::peripherals::pwm::PwmState::*;
                let word: Word = match state {
                    Disabled => 0,
                    Enabled(ref nzu8) => nzu8.get() as Word,
                };

                Ok(Self::with_value(word))
            }

            fn set<'a, I>(interp: &mut I, value: Word) -> WriteAttempt
            where
                I: InstructionInterpreterPeripheralAccess<'a>,
                <I as Deref>::Target: Peripherals<'a>,
            {
                use lc3_traits::peripherals::pwm::PwmState::*;
                use core::num::NonZeroU8;

                let state_val: u8 = value as u8;
                let state = match state_val {
                    0 => Disabled,
                    _ => Enabled(NonZeroU8::new(state_val).unwrap()),
                };

                Pwm::set_state(interp.get_peripherals_mut(), $pin, state);

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

            fn from<'a, I> (interp: &I) -> Result<Self, Acv> // TODO: change all these to some other kind of error since we already check for ACVs in read_word, etc.
            where
                I: InstructionInterpreterPeripheralAccess<'a>,
                <I as Deref>::Target: Peripherals<'a>,
            {
                let word = Pwm::get_duty_cycle(interp.get_peripherals(), $pin) as Word;

                Ok(Self::with_value(word))
            }

            fn set<'a, I>(interp: &mut I, value: Word) -> WriteAttempt
            where
                I: InstructionInterpreterPeripheralAccess<'a>,
                <I as Deref>::Target: Peripherals<'a>,
            {
                let duty_val: u8 = value as u8;

                Pwm::set_duty_cycle(interp.get_peripherals_mut(), $pin, duty_val);

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

            fn from<'a, I> (interp: &I) -> Result<Self, Acv>
            where
                I: InstructionInterpreterPeripheralAccess<'a>,
                <I as Deref>::Target: Peripherals<'a>,
            {
                let mode = Timers::get_mode(interp.get_peripherals(), $id);

                use lc3_traits::peripherals::timers::TimerMode::*;
                let word: Word = match mode {
                    SingleShot => 0,
                    Repeated => 1,
                };

                Ok(Self::with_value(word))
            }

            fn set<'a, I>(interp: &mut I, value: Word) -> WriteAttempt
            where
                I: InstructionInterpreterPeripheralAccess<'a>,
                <I as Deref>::Target: Peripherals<'a>,
            {
                use lc3_traits::peripherals::timers::TimerMode::*;
                let mode = if value.bit(0) {
                    Repeated
                } else {
                    SingleShot
                };

                Timers::set_mode(interp.get_peripherals_mut(), $id, mode);

                Ok(())
            }
        }

        impl Interrupt for $cr {
            const INT_VEC: u8 = $int_vec;
            const PRIORITY: u8 = TIMER_INT_PRIORITY;

            fn interrupt_ready<'a, I>(interp: &I) -> bool
            where
                I: InstructionInterpreterPeripheralAccess<'a>,
                <I as Deref>::Target: Peripherals<'a>,
            {
                Timers::interrupt_occurred(interp.get_peripherals(), $id)
            }

            fn interrupt_enabled<'a, I>(interp: &I) -> bool
            where
                I: InstructionInterpreterPeripheralAccess<'a>,
                <I as Deref>::Target: Peripherals<'a>
            {
                Timers::interrupts_enabled(interp.get_peripherals(), $id)
            }

            fn reset_interrupt_flag<'a, I>(interp: &mut I)
                where
                    I: InstructionInterpreterPeripheralAccess<'a>,
                    <I as Deref>::Target: Peripherals<'a>
            {
                if Timers::interrupts_enabled(interp.get_peripherals(), $id) {
                    Timers::reset_interrupt_flag(interp.get_peripherals_mut(), $id);
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

            fn from<'a, I> (interp: &I) -> Result<Self, Acv> // TODO: change all these to some other kind of error since we already check for ACVs in read_word, etc.
            where
                I: InstructionInterpreterPeripheralAccess<'a>,
                <I as Deref>::Target: Peripherals<'a>,
            {
                let state = Timers::get_state(interp.get_peripherals(), $id);

                use lc3_traits::peripherals::timers::TimerState::*;
                let value = match state {
                    Disabled => 0,
                    WithPeriod(period) => period.into(),
                };

                Ok(Self::with_value(value))
            }

            fn set<'a, I>(interp: &mut I, value: Word) -> WriteAttempt
            where
                I: InstructionInterpreterPeripheralAccess<'a>,
                <I as Deref>::Target: Peripherals<'a>,
            {
                use lc3_traits::peripherals::timers::TimerState::*;
                use lc3_traits::peripherals::timers::Period;
                let state = match value {
                    0 => Disabled,
                    nonzero => WithPeriod(Period::new(nonzero).unwrap()), // TODO: will this fail?
                };
                Timers::set_state(interp.get_peripherals_mut(), $id, state);

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

    pub fn set_priority<'a, I>(&mut self, interp: &mut I, priority: u8)
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
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

    fn set_mode<'a, I>(&mut self, interp: &mut I, user_mode: bool)
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
        self.0 = self.0.u16(0..14) | (Into::<Word>::into(user_mode) << 15);

        // Don't return a `WriteAttempt` since PSR accesses are infallible.
        self.write_current_value(interp).unwrap()
    }

    pub fn to_user_mode<'a, I>(&mut self, interp: &mut I)
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
        self.set_mode(interp, true)
    }

    pub fn to_privileged_mode<'a, I>(&mut self, interp: &mut I)
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
        self.set_mode(interp, false)
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

    pub fn set_cc<'a, I>(&mut self, interp: &mut I, word: Word)
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
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
    fn from<'a, I: InstructionInterpreterPeripheralAccess<'a>>(interp: &I) -> Result<Self, Acv>
    where
        <I as Deref>::Target: Peripherals<'a>,
    {
        Ok(Self::with_value(
            interp.get_word_force_memory_backed(Self::ADDR),
        ))
    }

    fn set<'a, I: InstructionInterpreterPeripheralAccess<'a>>(
        interp: &mut I,
        value: Word,
    ) -> WriteAttempt
    where
        <I as Deref>::Target: Peripherals<'a>,
    {
        interp.set_word_force_memory_backed(Self::ADDR, value);

        if !value.bit(15) {
            interp.halt();
        }

        Ok(())
    }
}

impl MemMappedSpecial for MCR {}

impl MCR {
    fn set_running_bit<'a, I>(&mut self, interp: &mut I, bit: bool)
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
        self.0 = (self.0 & (!WORD_MAX_VAL.select(15..15))) | ((bit as Word) << 15);

        // Don't return a `WriteAttempt` since MCR accesses don't produce ACVs (and are hence infallible).
        self.write_current_value(interp).unwrap();
    }

    pub fn is_running(&self) -> bool {
        self.0.bit(15)
    }

    pub fn halt<'a, I>(&mut self, interp: &mut I)
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
        self.set_running_bit(interp, false);
    }

    pub fn run<'a, I>(&mut self, interp: &mut I)
    where
        I: InstructionInterpreterPeripheralAccess<'a>,
        <I as Deref>::Target: Peripherals<'a>,
    {
        self.set_running_bit(interp, true);
    }
}
