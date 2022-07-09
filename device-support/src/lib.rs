//! Supporting materials for devices running the UTP LC-3 Simulator.
//!
//! TODO!

// TODO: forbid
#![warn(
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    legacy_directory_ownership,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    plugin_as_library,
    private_in_public,
    safe_extern_statics,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_lifetimes,
    unused_comparisons,
    unused_parens,
    while_true
)]
// TODO: deny
#![warn(
    missing_debug_implementations,
    intra_doc_link_resolution_failure,
    missing_docs,
    unsafe_code,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    rust_2018_idioms
)]
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
