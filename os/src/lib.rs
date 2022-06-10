//! A barebones base image for the LC-3 with some tests.
//!
//! TODO!

// The OS's `program! { }` macro needs this!
#![recursion_limit = "2048"]

#![doc(test(attr(deny(warnings))))]
#![doc(html_logo_url = "")] // TODO!

#![deny(rustdoc::broken_intra_doc_links)] // TODO: this is temporary

// Enable the `doc_cfg` feature when running rustdoc.
#![cfg_attr(all(docs, not(doctest)), feature(doc_cfg))]

// We're a no_std crate!
#![no_std]

extern crate static_assertions as sa;

// Note that these are 'page' aligned with the default starting sp pointing at
// the end of this page. The idea here is to minimize the number of pages that
// get modified (i.e. are dirty).

pub const USER_PROG_START_ADDR_SETTING_ADDR: lc3_isa::Addr = 0x0600;
pub const ERROR_ON_ACV_SETTING_ADDR: lc3_isa::Addr = 0x0601;
pub const OS_STARTING_SP_ADDR: lc3_isa::Addr = 0x0602;

pub const OS_DEFAULT_STARTING_SP: lc3_isa::Word = 0x0700;

mod os;

pub mod traps;

pub use os::{OS, OS_IMAGE};
