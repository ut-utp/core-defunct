//! TODO!

use core::mem::size_of;

use lc3_traits::control::rpc::{Transport, RequestMessage, ResponseMessage};

use crate::util::fifo;
use super::SENTINEL_BYTE;

// Check that CAPACITY is such that we can hold at least one full
// request/response:
sa::const_assert!(fifo::DEFAULT_CAPACITY >= (3 * size_of::<RequestMessage>()));
sa::const_assert!(fifo::DEFAULT_CAPACITY >= (3 * size_of::<ResponseMessage>()));

// pub mod multiplexed;
pub mod uart_simple;

using_std! {
    #[cfg_attr(all(docs, not(doctest)), doc(cfg(all(feature = "host_transport", not(target_arch = "wasm32")))))]
    #[cfg(all(feature = "host_transport", not(target_arch = "wasm32")))]
    pub mod uart_host;
}
