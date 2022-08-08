//! Functions and bits that are useful for testing the interpreter.

#![doc(test(attr(deny(rust_2018_idioms, warnings))))]
#![doc(html_logo_url = "")] // TODO!

// Enable the `doc_cfg` feature when running rustdoc.
#![cfg_attr(all(docs, not(doctest)), feature(doc_cfg))]

#[doc(no_inline)]
pub use {
    lc3_isa::{insn, Addr, Instruction, Reg, Word},
    lc3_shims::{
        memory::MemoryShim,
        peripherals::{
            PeripheralsShim, ShareablePeripheralsShim, SourceShim
        },
    },
    lc3_baseline_sim::interp::{
        Interpreter, InstructionInterpreterPeripheralAccess,
        InstructionInterpreter
    },
    lc3_traits::peripherals::PeripheralsWrapper,
    lc3_application_support::shim_support::new_shim_peripherals_set
};

#[doc(no_inline)]
pub use pretty_assertions::*;


mod runner;
#[macro_use] pub mod macros;
mod misc;

// The bash script will not work on Windows.
#[cfg_attr(all(docs, not(doctest)), doc(cfg(target_family = "unix")))]
#[cfg(target_family = "unix")]
#[macro_use] pub mod lc3tools;

pub use runner::*;
pub use misc::*;
