//! Example implementations of the LC-3 peripherals suitable for simulation.
//!
//! TODO!

#![doc(test(attr(deny(rust_2018_idioms, warnings))))]
#![doc(html_logo_url = "")] // TODO!

// Enable the `doc_cfg` feature when running rustdoc.
#![cfg_attr(all(docs, not(doctest)), feature(doc_cfg))]

extern crate static_assertions as sa;

#[doc(hidden)]
#[macro_export]
macro_rules! not_wasm {
    ($($i:item)*) => {
        $(
            #[cfg_attr(all(docs, not(doctest)), doc(cfg(not(target_arch = "wasm32"))))]
            #[cfg(not(target_arch = "wasm32"))]
            $i
        )*
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! wasm {
    ($($i:item)*) => {
        $(
            #[cfg_attr(all(docs, not(doctest)), doc(cfg(target_arch = "wasm32")))]
            #[cfg(target_arch = "wasm32")]
            $i
        )*
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! specialize {
    (   wasm: { $($wasm_item:item)+ }
        not: { $($other:item)+ }
    ) => {
        $crate::wasm! { $($wasm_item)+ }

        $crate::not_wasm! { $($other)+ }
    };
}

pub mod memory;
pub mod peripherals;
