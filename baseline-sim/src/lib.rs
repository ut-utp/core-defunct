//! An instruction level simulator for the LC-3.
//!
//! TODO!

#![doc(test(attr(deny(warnings))))]
#![doc(html_logo_url = "")] // TODO!
// TODO: add doc URL to all

// Enable the `doc_cfg` feature when running rustdoc.
#![cfg_attr(all(docs, not(doctest)), feature(doc_cfg))]

// Mark the crate as no_std if the `std` feature is **not** enabled.
#![cfg_attr(not(feature = "std"), no_std)]

extern crate static_assertions as sa;

pub mod interp;
pub mod mem_mapped;
pub mod sim;

pub use mem_mapped::*;
