//! TODO!

use crate::util::fifo;

use lc3_traits::control::rpc::{RequestMessage, ResponseMessage};

use core::mem::size_of;

// Check that CAPACITY is such that we can hold at least one full
// request/response:
sa::const_assert!(fifo::CAPACITY >= (3 * size_of::<RequestMessage>()));
sa::const_assert!(fifo::CAPACITY >= (3 * size_of::<ResponseMessage>()));

pub mod uart_simple;

using_std! {
    #[cfg_attr(all(docs, not(doctest)), doc(cfg(all(feature = "host_transport", not(target_arch = "wasm32")))))]
    #[cfg(all(feature = "host_transport", not(target_arch = "wasm32")))]
    pub mod uart_host;
}
