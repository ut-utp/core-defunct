//! [`Output` device trait](Output) and friends.

use core::fmt::{self, Display};

use serde::{Deserialize, Serialize};

#[ambassador::delegatable_trait]
pub trait Output {
    fn write_data(&mut self, c: u8) -> Result<(), OutputError>;

    // Gets set to high automagically when more data can be taken.
    // Gets set to low (by [write_data](Output::write_data)) when
    // data is being written.
    fn current_data_written(&self) -> bool;

    fn interrupt_occurred(&self) -> bool;
    fn reset_interrupt_flag(&mut self,);

    fn set_interrupt_enable_bit(&mut self, bit: bool);
    fn interrupts_enabled(&self) -> bool;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OutputError {
    NonUnicodeCharacter(u8),
    IoError,
    NotReady,
}

impl Display for OutputError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        use OutputError::*;

        match self {
            NonUnicodeCharacter(c) => write!(fmt, "Tried to write a non-unicode output: {:#2X}", c),
            IoError => write!(fmt, "I/O error when writing output"),
            NotReady => write!(fmt, "Attempted to write when the output was not ready")
        }
    }
}

using_std! {
    use std::io::Error;
    impl From<Error> for OutputError {
        fn from(_e: Error) -> OutputError {
            OutputError::IoError
        }
    }
}
