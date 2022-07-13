//! Types and friends for the LC-3 ISA.
//!
//! TODO!

#![doc(test(attr(deny(warnings))))]
#![doc(html_logo_url = "")] // TODO!

// Mark the crate as no_std if the `std` feature is **not** enabled.
#![cfg_attr(not(feature = "std"), no_std)]

// Enable the `doc_cfg` feature when running rustdoc.
#![cfg_attr(all(docs, not(doctest)), feature(doc_cfg))]

extern crate static_assertions as sa;

use core::mem::size_of;

/// Address type/size for the LC-3.
pub type Addr = u16;

/// Maximum possible address value.
pub const ADDR_MAX_VAL: Addr = Addr::max_value();

/// Word type/size for the LC-3.
pub type Word = u16;

/// Signed counterpart of the [`Word`] type.
pub type SignedWord = i16;

// Offer utility function for verifying bit lengths.
// Currently used for assembler.
pub use isa::check_signed_imm;

// Make sure our `Word` and `SignedWord` types are counterparts:
sa::const_assert!(size_of::<Word>() == size_of::<SignedWord>());

// And that `SignedWord` truly is signed:
sa::const_assert!({
    #[allow(trivial_numeric_casts)]
    ((-1) as SignedWord).is_negative()
});

pub const PSR: Addr = 0xFFFC;
pub const MCR: Addr = 0xFFFE;

pub const OS_START_ADDR: Addr = 0x0200; // TODO: should this go here?

pub const MEM_MAPPED_START_ADDR: Addr = 0xFE00;
pub const USER_PROGRAM_START_ADDR: Addr = 0x3000;

pub const TRAP_VECTOR_TABLE_START_ADDR: Addr = 0x0000;
pub const NUM_TRAP_VECTORS: Addr = 256;

pub const INTERRUPT_VECTOR_TABLE_START_ADDR: Addr = 0x0100;
// Exceptions: 0x0100 - 0x017F
pub const EXCEPTION_SERVICE_ROUTINES_START_ADDR: Addr = INTERRUPT_VECTOR_TABLE_START_ADDR;
pub const NUM_EXCEPTION_SERVICE_ROUTINES: Addr = 128;
// Interrupts: 0x0180 - 0x01FF
pub const INTERRUPT_SERVICE_ROUTINES_START_ADDR: Addr =
    INTERRUPT_VECTOR_TABLE_START_ADDR + NUM_EXCEPTION_SERVICE_ROUTINES;
pub const NUM_INTERRUPT_SERVICE_ROUTINES: Addr = 128;

pub const PRIVILEGE_MODE_VIOLATION_EXCEPTION_VECTOR: u8 = 0x00;
pub const ILLEGAL_OPCODE_EXCEPTION_VECTOR: u8 = 0x01;
pub const ACCESS_CONTROL_VIOLATION_EXCEPTION_VECTOR: u8 = 0x02; // TODO: Verify

/// Maximum possible word value.
pub const WORD_MAX_VAL: Word = Word::max_value();

/// Size of the LC-3 address space in [`Word`](Word)s.
pub const ADDR_SPACE_SIZE_IN_WORDS: usize = (ADDR_MAX_VAL as usize) + 1;

/// Size of the LC-3 address space in bytes.
pub const ADDR_SPACE_SIZE_IN_BYTES: usize = ADDR_SPACE_SIZE_IN_WORDS * size_of::<Word>();

mod fmt;
mod isa;
mod macros;
mod misc;

#[doc(hidden)]
pub use macros::overlap_error as _macro_support;

pub use isa::*;
pub use misc::util;
