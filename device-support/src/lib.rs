//! Supporting materials for devices running the UTP LC-3 Simulator.
//!
//! TODO!

#![doc(test(attr(deny(rust_2018_idioms, warnings))))]
#![doc(html_logo_url = "")] // TODO!

// Mark the crate as no_std if the `std` feature is *not* enabled.
#![cfg_attr(not(feature = "std"), no_std)]

// Enable the `doc_cfg` feature when running rustdoc.
#![cfg_attr(all(docs, not(doctest)), feature(doc_cfg))]

macro_rules! using_std { ($($i:item)*) => ($(
    #[cfg_attr(all(docs, not(doctest)), doc(cfg(feature = "std")))]
    #[cfg(feature = "std")]
    $i
)*) }
macro_rules! using_alloc { ($($i:item)*) => ($(
    #[cfg_attr(all(docs, not(doctest)), doc(cfg(feature = "alloc")))]
    #[cfg(feature = "alloc")]
    $i
)*) }

using_alloc! { #[allow(unused_extern_crates)] extern crate alloc; }

extern crate static_assertions as sa;


pub mod memory;

pub mod peripherals;

pub mod rpc;

pub mod util;
