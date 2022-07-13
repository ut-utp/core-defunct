//! Supporting materials for devices running the UTP LC-3 Simulator.
//!
//! TODO!

#![doc(test(attr(deny(rust_2018_idioms, warnings))))]
#![doc(html_logo_url = "")] // TODO!

// Enable the `doc_cfg` feature when running rustdoc.
#![cfg_attr(all(docs, not(doctest)), feature(doc_cfg))]

#[doc(hidden)]
#[macro_export]
macro_rules! not_wasm {
    ($($i:item)*) => {
        $(
            #[cfg_attr(all(docs, not(doctest)), doc(not(target_arch = "wasm32")))]
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
            #[cfg_attr(all(docs, not(doctest)), doc(target_arch = "wasm32"))]
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

pub mod event_loop;
pub mod init;
pub mod io_peripherals;
pub mod shim_support;
