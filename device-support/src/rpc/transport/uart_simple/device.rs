//! A primitive, simplistic, [`Transport`] impl that uses UART.
//!
//! This does not use interrupts or DMA and does not attempt to be zero-copy.

use super::{ConsumeData, SENTINEL_BYTE};
use crate::util::{Fifo, fifo};

use lc3_traits::control::rpc::Transport;
use lc3_traits::control::{Identifier, Version, version_from_crate};

use embedded_hal::serial::{Read, Write};
use nb::block;

use core::cell::RefCell;
use core::fmt::Debug;
use core::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug)]
pub struct UartTransport<R: Read<u8>, W: Write<u8>, const RECV_LEN: usize = { fifo::DEFAULT_CAPACITY }>
where
    <R as Read<u8>>::Error: Debug,
    <W as Write<u8>>::Error: Debug,
{
    innards: RefCell<Innards<RECV_LEN, R, W>>,
    recv_error_count: AtomicU32,
}

// So that we only have _one_ RefCell to borrow.
#[derive(Debug)]
struct Innards<const RECV_LEN: usize, R, W> {
    read: R,
    write: W,
    internal_buffer: Fifo<u8, RECV_LEN>,
}

impl<R: Read<u8>, W: Write<u8>, const RECV_LEN: usize> UartTransport<R, W, RECV_LEN>
where
    <R as Read<u8>>::Error: Debug,
    <W as Write<u8>>::Error: Debug,
{
    // Can't be const until bounds are allowed.
    pub const fn new(read: R, write: W) -> Self {
        Self {
            innards: RefCell::new(Innards {
                read: read,
                write: write,
                internal_buffer: Fifo::new(),
            }),
            recv_error_count: AtomicU32::new(0),
        }
    }

    // Just a helper that increments the error counter:
    fn err<T>(&self, t: T) -> T {
        self.recv_error_count.fetch_add(1, Ordering::Relaxed);
        t
    }

    fn recv_inner(&self, blocking: bool) -> Result<Fifo<u8, RECV_LEN>, Option<R::Error>> {
        let mut this = self.innards.borrow_mut();
        use nb::Error::*;

        loop {
            match this.read.read() {
                Ok(word) => {
                    if word == SENTINEL_BYTE {
                        // 0 is the sentinel!
                        break Ok(core::mem::replace(&mut this.internal_buffer, Fifo::<u8, RECV_LEN>::new()))
                    } else {
                        // TODO: don't panic here, just dump the buffer or
                        // something.
                        this.internal_buffer.push(word).unwrap()
                    }
                },
                Err(WouldBlock) => {
                    if blocking {
                        continue
                    } else {
                        break Err(None)
                    }
                },
                Err(Other(err)) => {
                    break self.err(Err(Some(err)))
                }
            }
        }
    }
}

impl<const RECV_LEN: usize, R: Read<u8>, W: Write<u8>, SendFormat: ConsumeData> Transport<SendFormat, Fifo<u8, RECV_LEN>> for UartTransport<R, W, RECV_LEN>
where
    <R as Read<u8>>::Error: Debug,
    <W as Write<u8>>::Error: Debug,
{
    type RecvErr = R::Error;
    type SendErr = W::Error;

    const ID: Identifier = Identifier::new_from_str_that_crashes_on_invalid_inputs("UART");
    const VER: Version = {
        let ver = version_from_crate!();

        let id = Identifier::new_from_str_that_crashes_on_invalid_inputs("ehal");

        Version::new(ver.major, ver.minor, ver.patch, Some(id))
    };

    fn send(&self, mut message: SendFormat) -> Result<(), W::Error> {
        let mut this = self.innards.borrow_mut();
        let write = &mut this.write;

        message.consume(|byte| {
            block!(write.write(byte))
        })?;

        block!(write.flush())
    }

    fn get(&self) -> Result<Fifo<u8, RECV_LEN>, Option<R::Error>> {
        self.recv_inner(false)
    }

    fn blocking_get(&self) -> Result<Fifo<u8, RECV_LEN>, Option<Self::RecvErr>> {
        self.recv_inner(true)
    }

    fn num_get_errors(&self) -> u64 {
        self.recv_error_count.load(Ordering::Relaxed) as u64
    }
}
