//! A set of transport impls that also implement multiplexing and can be used
//! with the companion [`Input`/`Output` peripheral trait
//! implementations](crate::peripherals::multiplexed_io).
//!
//! Like the [simple transport impls](super::uart_simple), performance isn't a
//! priority for these impls, but simplicity and running on different devices
//! without any fuss _is_.

// TODO: checksum, len, etc.

//! ## Transmission Scheme
//!
/*! Device:
```ignore
type Inp
 = Control Cobs RequestMsg Zero
 | Input Byte Zero

type Out
 = Control Cobs ResponseMsg Zero
 | Output Byte Zero
```
*/

/*! Host:
```ignore
type Inp
 = Control Cobs ResponseMsg Zero
 | Output Byte Zero

type Out
 = Control Cobs InputMsg Zero
 | Input Byte Zero
```
*/

use core::{cell::RefCell, marker::PhantomData, sync::atomic::{Ordering, AtomicU32}, str::Utf8Error};

use lc3_traits::control::{Identifier, Version};

use crate::{util::fifo::Fifo, rpc::encoding::DynFifoBorrow};

use self::msg_slugs::{CONTROL_MSG_SLUG, IO_MSG_SLUG};

use super::*;

pub mod msg_slugs {
    pub const CONTROL_MSG_SLUG: u8 = 'C' as u8;
    pub const IO_MSG_SLUG: u8 = 'I' as u8;
}

pub trait MultiplexedTransport<S, R>: Transport<S, R> {
    /// Immediately pushes out the output character.
    ///
    /// Returns a `SendErr` if there are errors in transmission.
    fn put_output(&self, out: char) -> Result<(), <Self as Transport<S, R>>::SendErr>;

    /// Convenience method; allows for implementations to specialize, if
    /// possible.
    fn put_string(&self, s: &str) -> Result<(), <Self as Transport<S, R>>::SendErr> {
        for c in s.chars() {
            self.put_output(c)?;
        }

        Ok(())
    }

    /// Tries to get the current pending character.
    ///
    /// Returns `None` if there is no pending character.
    ///
    /// Note: this interface suggests that implementors only hold on to the most
    /// recent character; if multiple characters pile up, they will be lost.
    /// This isn't great but our hands are tied here; this is what the LC-3
    /// intends for the `Input` device.
    ///
    /// That said, implementors are free to buffer multiple character if they so
    /// choose.
    fn get_input(&self) -> Option<char>;

    /// `true` if there are one or more pending input characters, false
    /// otherwise.
    fn has_pending_input(&self) -> bool;
}

// This is not a great solution; it assumes that the underlying transport layer
// does COBS-style sentinel based framing.
//
// It also fixes on using `Fifo<u8>`..
//
// We should maybe split out framing from actual byte transport? but then we
// lose our ability to support higher level protocols easily.

/// Wraps a [`Transport`] impl that does zero-sentinel based framing but only
/// on **recieved messages** (assumes that `Inner::send` just sends the given
/// bytes and does not append anything).
///
/// Assumes that messages received on the wire will be prefixed with a [message
/// slug byte](msg_slugs).
///
/// `send`:
///   - forwards to `Inner::send`
///   - calls to [`MultiplexedTransport::put_output`] will also call
///     `Inner::send`
///
/// `get`:
///   - intercepts calls to `Inner::send` to see if a framed message is actually
///     an [I/O message](msg_slugs::IO_MSG_SLUG)
///   - other messages are forwarded along, minus the slug
pub struct Multiplexed<SendFormatConstructor, S, /* R, */const RECV_LEN: usize, Inner, const INPUT_BUF_LEN: usize = 32>
where
    SendFormatConstructor: Fn() -> S,

    // Need to be able to send I/O messages:
    S: Extend<u8> + ExtendLimit, // i.e. encode

    // Want to be able to inspect what's received for I/O messages and slugs.
    // We also need to remove the slug before forwarding the message along:
    // R: AsRef
    // Just fixing on `Fifo<u8>` for now...

    // We want to be able to send `S` and recieve `Fifo`s:
    Inner: Transport<S, Fifo<u8, RECV_LEN>>,

    // But we'd also like to be able to just send regular old slices:
    Inner: for<'a> Transport<
        &'a [u8],
        Fifo<u8, RECV_LEN>,
        SendErr = <Inner as Transport<S, Fifo<u8, RECV_LEN>>>::SendErr,
    >,
{
    inner: Inner,
    pending_chars: RefCell<Fifo<char, INPUT_BUF_LEN>>,
    send_ctor: SendFormatConstructor,
    recv_error_count: AtomicU32,
    _p: PhantomData<S>,
}

impl<SendCtor, S, const RECV_LEN: usize, Inner, const INP_LEN: usize> Multiplexed<SendCtor, S, RECV_LEN, Inner, INP_LEN>
where
    SendCtor: Fn() -> S,
    S: Extend<u8> + ExtendLimit,
    Inner: Transport<S, Fifo<u8, RECV_LEN>>,
    Inner: for<'a> Transport<
        &'a [u8],
        Fifo<u8, RECV_LEN>,
        SendErr = <Inner as Transport<S, Fifo<u8, RECV_LEN>>>::SendErr,
    >,
{
    pub const fn new(inner: Inner, send_ctor: SendCtor) -> Self {
        Self {
            inner,
            pending_chars: RefCell::new(Fifo::new()),
            send_ctor,
            recv_error_count: AtomicU32::new(0),
            _p: PhantomData,
        }
    }

    // Just a helper that increments the error counter:
    fn err<T>(&self, t: T) -> T {
        self.recv_error_count.fetch_add(1, Ordering::Relaxed);
        t
    }

    fn post_process_received_output(
        &self,
        mut recv: Fifo<u8, RECV_LEN>,
    ) -> Result<
        Fifo<u8, RECV_LEN>,
        Option<
            MultiplexedRecvError<
                <Inner as Transport<S, Fifo<u8, RECV_LEN>>>::RecvErr
            >
        >
    > {
        if recv.is_empty() {
            return self.err(Err(Some(MultiplexedRecvError::EmptyMessageFromInnerTransport)))
        }

        match recv.pop().unwrap() {
            CONTROL_MSG_SLUG => Ok(recv),
            IO_MSG_SLUG => {
                if recv.len() == 4 {
                    let (a, b) = recv.as_slices();

                    let mut arr;
                    let arr: &[u8; 4] = if a.len() == 4 && b.len() == 0 {
                        a.try_into().unwrap()
                    } else {
                        // Gotta make a copy:
                        arr = [0; 4];

                        for (i, c) in a.iter().chain(b.iter()).enumerate() {
                            arr[i] = *c;
                        }

                        &arr
                    };

                    match char_from_bytes(arr) {
                        Ok(c) => {
                            let mut inp_buf = self.pending_chars.borrow_mut();
                            if let Err(()) = inp_buf.push(c) {
                                /* uh-oh, have to drop this character :( */
                                log::warn!("input buffer full; dropped input char: {}", c);
                            }

                            Err(None)
                        },
                        Err(char_decode_err) => self.err(Err(Some(
                            MultiplexedRecvError::CharacterDecodeError(
                                char_decode_err
                            )
                        ))),
                    }
                } else {
                    self.err(Err(Some(MultiplexedRecvError::InvalidIoMsgLength { length: recv.len() })))
                }
            },
            other => self.err(Err(Some(MultiplexedRecvError::UnknownSlug { got: other }))),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum MultiplexedRecvError<Inner> {
    EmptyMessageFromInnerTransport,
    UnknownSlug { got: u8 },
    InvalidIoMsgLength { length: usize },
    CharacterDecodeError(Option<Utf8Error>),
    InnerError(Inner),
}

impl<Inner> From<Inner> for MultiplexedRecvError<Inner> {
    fn from(i: Inner) -> Self {
        MultiplexedRecvError::InnerError(i)
    }
}
// TODO: thiserror, derive Error

// Note: we cannot log from inside the inner impl if we plan to expose a logging
// backend that uses this type to send log messages since we'd then attempt to
// borrow the inner impl's RefCell mutably multiple times and would crash.

impl<SendCtor, S, const RECV_LEN: usize, Inner, const INP_LEN: usize> Transport<S, Fifo<u8, RECV_LEN>>
    for Multiplexed<SendCtor, S, RECV_LEN, Inner, INP_LEN>
where
    SendCtor: Fn() -> S,
    S: Extend<u8> + ExtendLimit,
    Inner: Transport<S, Fifo<u8, RECV_LEN>>,
    Inner: for<'a> Transport<
        &'a [u8],
        Fifo<u8, RECV_LEN>,
        SendErr = <Inner as Transport<S, Fifo<u8, RECV_LEN>>>::SendErr,
    >,
{
    type RecvErr = MultiplexedRecvError<
        <Inner as Transport<S, Fifo<u8, RECV_LEN>>>::RecvErr
    >;
    type SendErr = <Inner as Transport<S, Fifo<u8, RECV_LEN>>>::SendErr;

    const ID: Identifier = {
        let base = <Inner as Transport<S, Fifo<u8, RECV_LEN>>>::ID.as_array();

        Identifier::new_that_crashes_on_invalid_inputs([
            b'm', base[0], base[2], base[3],
        ])
    };
    const VER: Version = {
        let ver = lc3_traits::version_from_crate!();
        let id = <Inner as Transport<S, Fifo<u8, RECV_LEN>>>::VER.pre;

        let id = if id.is_some() {
            id
        } else {
            Some(Identifier::new_from_str_that_crashes_on_invalid_inputs("mplx"))
        };

        Version::new(ver.major, ver.minor, ver.patch, id)
    };

    // Send a Control message:
    fn send(&self, message: S) -> Result<(), Self::SendErr> {
        self.inner.send([CONTROL_MSG_SLUG].as_slice())?;
        self.inner.send(message)
    }

    fn get(&self) -> Result<Fifo<u8, RECV_LEN>, Option<Self::RecvErr>> {
        let recv = Transport::<S, _>::get(&self.inner).map_err(|i| i.map(Into::into))?;
        self.post_process_received_output(recv)
    }

    fn blocking_get(&self) -> Result<Fifo<u8, RECV_LEN>, Option<Self::RecvErr>> { // TODO: &mut ?
        let recv = Transport::<S, _>::blocking_get(&self.inner).map_err(|i| i.map(Into::into))?;
        self.post_process_received_output(recv)
    }

    fn num_get_errors(&self) -> u64 {
        Transport::<S, _>::num_get_errors(&self.inner) + (self.recv_error_count.load(Ordering::Relaxed) as u64)
    }
}

impl<SendCtor, S, const RECV_LEN: usize, Inner, const INP_LEN: usize> MultiplexedTransport<S, Fifo<u8, RECV_LEN>>
    for Multiplexed<SendCtor, S, RECV_LEN, Inner, INP_LEN>
where
    SendCtor: Fn() -> S,
    S: Extend<u8> + ExtendLimit,
    Inner: Transport<S, Fifo<u8, RECV_LEN>>,
    Inner: for<'a> Transport<
        &'a [u8],
        Fifo<u8, RECV_LEN>,
        SendErr = <Inner as Transport<S, Fifo<u8, RECV_LEN>>>::SendErr,
    >,
{
    fn put_output(&self, out: char) -> Result<(), <Self as Transport<S, Fifo<u8, RECV_LEN>>>::SendErr> {
        // For short messages like these, don't bother with `SendFormat` and the
        // reusable buffer that `SendCtor` may give us.
        //
        // Instead just construct our own exactly sized array and send that:
        let mut buf = [0; 6];
        let encoded = char_to_bytes(out);

        /* SLUG [ bytes ] SENTINEL */
        for (i, c) in [IO_MSG_SLUG].into_iter().chain(encoded).chain([SENTINEL_BYTE]).enumerate() {
            buf[i] = c;
        }

        self.inner.send(buf.as_slice())
    }

    fn get_input(&self) -> Option<char> {
        let mut fifo = self.pending_chars.try_borrow_mut().ok()?;
        fifo.pop()
    }

    fn has_pending_input(&self) -> bool {
        self.pending_chars.try_borrow().map(|f| !f.is_empty()).unwrap_or(false)
    }

    fn put_string(&self, s: &str) -> Result<(), <Self as Transport<S, Fifo<u8, RECV_LEN>>>::SendErr> {
        // For longer messages we can and will use the `SendCtor` provided
        // buffer.
        //
        // The tricky bit here is figuring our whether or not our string is long
        // enough that we need to split it up into chunks to avoid
        // _overwhelming_ the underlying buffer that `SendCtor` will give us.
        //
        // Ideally we'd have an interface like `Write` or `Fifo::push_iter` that
        // tell us how much of the buffer was _not_ read but unfortunately
        // `Write` is not in `core` and we'd rather not fix on using `Fifo` in
        // this impl.
        //
        // So, we defined our own trait for this, for now.
        #[allow(unused_parens)]
        let single_char_encoded_length = (
            1 + // slug
            4 + // encoded character size
            1   // sentinel
        );

        let mut it = s.chars().peekable();
        while it.peek().is_some() {
            let mut buf = (self.send_ctor)();

            // How many chars do we have room for?
            let max_acceptable = buf
                .limit()
                .map(|bytes| bytes / single_char_encoded_length /* div_floor */)
                .unwrap_or(usize::MAX);

            let chunk = it.clone().take(max_acceptable);
            let bytes = chunk.flat_map(|char| {
                [IO_MSG_SLUG].into_iter().chain(char_to_bytes(char)).chain([SENTINEL_BYTE])
            });
            buf.extend(bytes);

            self.inner.send(buf)?;

            // It feels dumb that we have to do this iteration twice and that
            // there isn't a way for `take` or a similar combinator to give us
            // the _rest_ of the iterator...
            // it.advance_by(max_acceptable); /* unstable */
            for _ in 0..max_acceptable {
                if it.next().is_none() {
                    break
                }
            }
        }

        Ok(())
    }
}

/// So that we can know how to chunk up strings before passing them into the
/// `SendCtor` provided storage thing.
///
/// Kind of a stopgap solution.
///
/// Ideally we'd have an interface like `Write` or `Fifo::push_iter` that tell
/// us how much of the buffer was _not_ read but unfortunately `Write` is not in
/// `core` and we'd rather not fix on using `Fifo` in this impl.
///
/// See [`Multiplexed::put_string`].
pub trait ExtendLimit: Extend<u8> {
    /// How much space do you have? How big of a thing can we shove into
    /// [`Extend::extend`] without it panicking?
    ///
    /// `None` means: "we allocate internally, do your worst"
    fn limit(&self) -> Option<usize> { None }
}

impl<const L: usize> ExtendLimit for Fifo<u8, L> {
    fn limit(&self) -> Option<usize> {
        Some(self.remaining())
    }
}

impl<'f, const L: usize> ExtendLimit for DynFifoBorrow<'f, L> {
    fn limit(&self) -> Option<usize> { (*self.0).limit() }
}

using_std! {
    impl ExtendLimit for std::vec::Vec<u8> { }
    impl ExtendLimit for std::collections::VecDeque<u8> { }
}

#[derive(Debug)]
pub struct MultiplexedTransportAsWriteImplAdapter<'m, S, R, Mt: ?Sized + MultiplexedTransport<S, R>> {
    inner: &'m Mt,
    _p: PhantomData<(S, R)>,
}

impl<'m, S, R, Mt: MultiplexedTransport<S, R>> core::fmt::Write for MultiplexedTransportAsWriteImplAdapter<'m, S, R, Mt> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.inner.put_string(s).map_err(|_| core::fmt::Error)
    }

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        self.inner.put_output(c).map_err(|_| core::fmt::Error)
    }
}

pub trait MultiplexedTransportExt<S, R>: MultiplexedTransport<S, R> {
    fn as_writeable(&self) -> /* impl core::fmt::Write */ MultiplexedTransportAsWriteImplAdapter<'_, S, R, Self> {
        MultiplexedTransportAsWriteImplAdapter { inner:  self, _p: PhantomData }
    }
}

impl<S, R, Mt: MultiplexedTransport<S, R>> MultiplexedTransportExt<S, R> for Mt { }

// #[derive(Debug)]
// enum State {
//     Clean,
//     AwaitingIoEnd { idx: usize, bytes: [u8; 4] },
//     AwaitingControlMsgEnd { buffer: Fifo<u8> },
// }

// enum Item {
//     ControlMsg(Fifo<u8>),
//     Io(char),
// }

// impl State {
//     const fn new() -> Self { State::Clean }

//     #[inline]
//     fn update(&mut self, byte: u8) -> Option<Item> {
//         use State::*;
//         match self {
//             Clean => {
//                 *self = match byte {
//                     CONTROL_MSG_SLUG => AwaitingControlMsgEnd { buffer: Fifo::new() },
//                     IO_MSG_SLUG => AwaitingIoEnd { idx: 0, bytes: [0; 4] },
//                     _ => {
//                         log::error!("Unknown message slug ({:#2X})! Ignoring...", byte);
//                         Clean
//                     },
//                 };

//                 None
//             },

//             AwaitingIoEnd { idx, bytes } => {
//                 bytes[*idx] = byte;
//                 *idx += 1;

//                 match idx {
//                     1..=3 => None,
//                     4 => {
//                         let res = char_from_bytes(bytes);
//                         if let None = res {
//                             log::error!("Invalid character received ({:?})! Ignoring...", bytes);
//                         }

//                         *self = Clean;
//                         res.map(Item::Io)
//                     },
//                     // Blocked on #37854, #67264
//                     // 0 | 5.. => unreachable!(),
//                     // 0 | 5..=usize::max_value() => unreachable!(),
//                     _ => unreachable!(),
//                 }
//             },

//             AwaitingControlMsgEnd { buffer } => {
//                 if byte == 0 {
//                     // Unfortunately rustc can't yet tell this is irrefutable:
// /*                    let AwaitingControlMsgEnd { buffer } = core::mem::replace(self, Clean);

//                     Some(Item::ControlMsg(buffer))*/

//                     // If we've got a sentinel, this is an Item!
//                     if let AwaitingControlMsgEnd { buffer } = core::mem::replace(self, Clean) {
//                         Some(Item::ControlMsg(buffer))
//                     } else {
//                         unreachable!()
//                     }
//                 } else {
//                     if let Err(()) = buffer.push(byte) {
//                         // If the buffer is full, log an error but don't reset
//                         // the buffer. This message is toast but we'll still
//                         // wait until a sentinel so that we'll be realigned.
//                         //
//                         // Ultimately when the sentinel does come along we'll
//                         // produce an Item that we _know_ is bad but this is
//                         // okay; the Controller/Device should be able to cope
//                         // with the resulting deserialization error.
//                         log::error!("Buffer is full but we got another byte for the current Control Message!");
//                     }

//                     None
//                 }
//             }
//         }
//     }
// }

// impl Default for State {
//     fn default() -> Self { Self::new() }
// }

// No need to worry about endianness for UTF-8.
fn char_to_bytes(c: char) -> [u8; 4] {
    let mut buf = [0; 4];

    c.encode_utf8(&mut buf);
    buf
}

fn char_from_bytes(bytes: &[u8; 4]) -> Result<char, Option<Utf8Error>> {
    core::str::from_utf8(bytes).map_err(Some)?.chars().next().ok_or(None)
}

// pub mod uart_device;
// pub use uart_device::MultiplexedUartTransport;

// using_std! {
//     #[cfg(all(feature = "host_transport", not(target_arch = "wasm32")))]
//     pub mod uart_host;
//     // #[cfg(all(feature = "host_transport", not(target_arch = "wasm32")))]
//     // pub use uart_host::MultiplexedHostUartTransport;
// }

#[cfg(test)]
mod char_enc_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn roundtrip(c: char) {
        assert_eq!(c, char_from_bytes(&char_to_bytes(c)).unwrap())
    }

    #[test]
    fn panagram() {
        for c in "Sphinx of black quartz, judge my vow.".chars() {
            roundtrip(c)
        }
    }

    #[test]
    fn emoji() {
        for e in "😄😀🙂😕☹️😴❓🤗🦀❤️".chars() {
            roundtrip(e)
        }
    }
}
