//! The [`Control` trait](crate::control::Control) and friends.
//!
//! Unlike the [`Peripherals` trait](crate::peripherals::Peripherals) and the
//! [`Memory` trait](crate::memory::Memory), there is no shim implementation of
//! Control; instead the 'shim' is an instruction level simulator that lives in
//! the [interp module](crate::interp).

use crate::error::Error;
use crate::memory::MemoryMiscError;
use crate::peripherals::adc::{AdcPinArr, AdcReadError, AdcState};
use crate::peripherals::gpio::{GpioPinArr, GpioReadError, GpioState};
use crate::peripherals::pwm::{PwmPinArr, PwmState};
use crate::peripherals::timers::{TimerArr, TimerState};

use lc3_isa::{Addr, Reg, Word, PSR};

use core::future::Future;

use serde::{Deserialize, Serialize};

pub const MAX_BREAKPOINTS: usize = 10;
pub const MAX_MEMORY_WATCHPOINTS: usize = 10;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Event {
    Breakpoint { addr: Addr },
    MemoryWatch { addr: Addr, data: Word },
    Interrupted, // If we get paused or stepped, this is returned.
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum State {
    Paused,
    RunningUntilEvent,
    Halted,
}

pub trait Control {
    type EventFuture: Future<Output = (Event, State)>;

    fn get_pc(&self) -> Addr;
    fn set_pc(&mut self, addr: Addr); // Should be infallible.

    fn get_register(&self, reg: Reg) -> Word;
    fn set_register(&mut self, reg: Reg, data: Word); // Should be infallible.

    fn get_registers_psr_and_pc(&self) -> ([Word; Reg::NUM_REGS], Word, Word) {
        let mut regs = [0; Reg::NUM_REGS];

        Reg::REGS
            .iter()
            .enumerate()
            .for_each(|(idx, r)| regs[idx] = self.get_register(*r));

        (regs, self.read_word(PSR), self.get_pc())
    }

    fn read_word(&self, addr: Addr) -> Word;
    fn write_word(&mut self, addr: Addr, word: Word);
    fn commit_memory(&mut self) -> Result<(), MemoryMiscError>;

    fn set_breakpoint(&mut self, addr: Addr) -> Result<usize, ()>;
    fn unset_breakpoint(&mut self, idx: usize) -> Result<(), ()>;
    fn get_breakpoints(&self) -> [Option<Addr>; MAX_BREAKPOINTS];
    fn get_max_breakpoints(&self) -> usize {
        MAX_BREAKPOINTS
    }

    fn set_memory_watchpoint(&mut self, addr: Addr) -> Result<usize, ()>;
    fn unset_memory_watchpoint(&mut self, idx: usize) -> Result<(), ()>;
    fn get_memory_watchpoints(&self) -> [Option<(Addr, Word)>; MAX_MEMORY_WATCHPOINTS];
    fn get_max_memory_watchpoints(&self) -> usize {
        MAX_MEMORY_WATCHPOINTS
    }

    // Execution control functions:
    fn run_until_event(&mut self) -> Self::EventFuture; // Can be interrupted by step or pause.
    fn step(&mut self) -> State;
    fn pause(&mut self); // TODO: should we respond saying whether or not the pause actually did anything (i.e. if we were already paused... it did not).

    fn get_state(&self) -> State;

    fn reset(&mut self);

    // TBD whether this is literally just an error for the last step or if it's the last error encountered.
    // If it's the latter, we should return the PC value when the error was encountered.
    //
    // Leaning towards it being the error in the last step though.
    fn get_error(&self) -> Option<Error>;

    // I/O Access:
    // TODO!! Does the state/reading separation make sense?
    fn get_gpio_states(&self) -> GpioPinArr<GpioState>;
    fn get_gpio_readings(&self) -> GpioPinArr<Result<bool, GpioReadError>>;
    fn get_adc_states(&self) -> AdcPinArr<AdcState>;
    fn get_adc_readings(&self) -> AdcPinArr<Result<u8, AdcReadError>>;
    fn get_timer_states(&self) -> TimerArr<TimerState>;
    fn get_timer_config(&self) -> TimerArr<Word>; // TODO: represent with some kind of enum? Word is problematic since it leaks interpreter impl details.
    fn get_pwm_states(&self) -> PwmPinArr<PwmState>;
    fn get_pwm_config(&self) -> PwmPinArr<u8>; // TODO: ditto with using u8 here; probably should be some kind of enum (the conflict is then we're kinda pushing implementors to represent state a certain way.. or at least to have to translate it to our enum).
    fn get_clock(&self) -> Word;

    // So with some of these functions that are basically straight wrappers over their Memory/Peripheral trait counterparts,
    // we have a bit of a choice. We can make Control a super trait of those traits so that we can have default impls of said
    // functions or we can make the implementor of Control manually wrap those functions.
    //
    // The downside to the super trait way is that it's a little weird; it requires that one massive type hold all the state
    // for all the Peripherals and Memory (and whatever the impl for Control ends up needing). You can of course store the
    // state for those things in their own types within your big type, but then to impl, say, Memory, you'd have to manually
    // pass all the calls along meaning we're back where we started.
    //
    // Associated types really don't seem to save us here (still gotta know where the state is stored which we don't know
    // when writing a default impl) and I can't think of a way that's appreciably better so I think we just have to eat it.
    //
    // We kind of got around this with the `PeripheralSet` struct in the peripherals module, but I'm not sure it'd work here.
}