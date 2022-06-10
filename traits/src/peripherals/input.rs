//! [`Input` device trait](Input) and related things.

use core::fmt::{self, Display};

use serde::{Deserialize, Serialize};

#[ambassador::delegatable_trait]
pub trait Input {
    // Warning! This is stateful!! It marks the current data as read.
    //
    // Also note: this is technically infallible (it's up to the
    // interpreter what to do for some of the edge cases, but
    // we'll presumably just return some default value) but since
    // we're letting the interpreter decide we *do* return a Result
    // type here.
    //
    // Must use interior mutability.
    fn read_data(&self) -> Result<u8, InputError>;
    // TODO: why not allow ^ to take `&mut self`?

    // TODO: maybe introduce a "no perturb read" function?
    // and an equivalent abstraction in `mem_mapped`?
    //
    // or just: have some mem_mapped devices have a "stateful_read = true" field
    // which causes reads of them to emit
    // 0s (or a "cannot read" error; we'd need to modify Control to support this but I think this is the way to go!)


    // this is a separate method than `read_data` because for interrupts we want
    // to be able to ask if there is any pending data without actually
    // _consuming_ it
    fn current_data_unread(&self) -> bool;

    fn interrupt_occurred(&self) -> bool;
    fn reset_interrupt_flag(&mut self,);

    fn set_interrupt_enable_bit(&mut self, bit: bool);
    fn interrupts_enabled(&self) -> bool;
}

// TODO: have something like this in stubs for this module

/*
struct AlwaysHasData<const D: u8> {
    interrupt_enabled: bool,
}
   impl<const D: u8> Input for AlwaysHasData<D> {
    fn read_data(&mut self) -> Result<u8, InputError> {
        Ok(D)
    }
    fn current_data_unread(&self) -> bool {
        true
    }
    fn register_interrupt_flag(&mut self,flag: & 'a AtomicBool) {
        todo!()
    }
    fn interrupt_occurred(&self) -> bool {
        true
    }
    fn reset_interrupt_flag(&mut self,) {
        /* we always have more data ready so we're always producing another interrupt, if interrupts are enabled */
    }
    fn set_interrupt_enable_bit(&mut self,bit:bool) {
        self.interrupt_enabled = bit;
    }
    fn interrupts_enabled(&self) -> bool {
        self.interrupt_enabled
    }
 */
// TODO: provide embedded_hal based Input/Output impls? (not multiplexed)

// TODO: pretty sure that if you don't read KBDR in your input interrupt ISR
// the interrupt will keep firing (i.e. the ready bit will remain high).
//
// should test this against lc3tools

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputError {
    NonUnicodeCharacter(u8), // TODO: should this be a u32?
    IoError,
    NoDataAvailable,
}

impl Display for InputError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        use InputError::*;

        match self {
            NonUnicodeCharacter(c) => write!(fmt, "Tried to read a non-unicode input: {:#2X}", c),
            IoError => write!(fmt, "I/O error when reading input"),
            NoDataAvailable => write!(fmt, "Attempted to read when no data had been inputted"),
        }
    }
}

using_std! {
    use std::io::Error;
    impl From<Error> for InputError {
        fn from(_e: Error) -> InputError {
            InputError::IoError
        }
    }
}
