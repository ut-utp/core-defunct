//! Traits defining the LC-3's peripherals, memory, and control interface.
//!
//! TODO!

#![doc(test(attr(deny(rust_2018_idioms, warnings))))]
#![doc(html_logo_url = "")] // TODO!

// Enable the `doc_cfg` feature when running rustdoc.
#![cfg_attr(all(docs, not(doctest)), feature(doc_cfg))]

// Mark the crate as no_std if the `std` feature **not** is enabled.
#![cfg_attr(not(feature = "std"), no_std)]

macro_rules! using_std { ($($i:item)*) => ($(
    #[cfg_attr(all(docs, not(doctest)), doc(cfg(feature = "std")))]
    #[cfg(feature = "std")]
    $i
)*) }

macro_rules! not_wasm { ($($i:item)*) => ($(#[cfg(not(target_arch = "wasm32"))]$i)*) }
macro_rules! wasm { ($($i:item)*) => ($(#[cfg(target_arch = "wasm32")]$i)*) }

extern crate static_assertions as sa;

pub mod error;

pub mod control;
pub mod memory;
pub mod peripherals;
