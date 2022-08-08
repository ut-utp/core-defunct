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

    // this is a separate method than `read_data` because for interrupts we want
    // to be able to ask if there is any pending data without actually
    // _consuming_ it
    fn current_data_unread(&self) -> bool;

    fn interrupt_occurred(&self) -> bool;
    fn reset_interrupt_flag(&mut self,);

    fn set_interrupt_enable_bit(&mut self, bit: bool);
    fn interrupts_enabled(&self) -> bool;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputError {
    NonUnicodeCharacter(u8),
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
