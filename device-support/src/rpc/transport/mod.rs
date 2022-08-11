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
pub mod multiplexed;

// TODO: maybe look into stripping all of this out in favor of `Read`/`Write`
// trait based abstractions (using `not_io` for no_std)?
//
// or at least at splitting the transport (stream of bytes) from the framing
// properly
