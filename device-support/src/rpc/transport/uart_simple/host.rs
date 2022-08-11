//! UART transport for computers.

use crate::util::Fifo;

use lc3_traits::control::rpc::Transport;
use lc3_traits::control::{version_from_crate, Identifier, Version};

use serialport::{
    DataBits, FlowControl, Parity, Result as SerialResult, SerialPort,
    StopBits,
};
pub use serialport::SerialPortBuilder;

use std::{
    borrow::Cow,
    cell::RefCell,
    io::{Error, ErrorKind, Read, Result as IoResult, Write},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

// TODO: Debug impl
pub struct HostUartTransport {
    serial: RefCell<Box<dyn SerialPort>>, // TODO: get rid of RefCell/trait object?
    internal_buffer: RefCell<Fifo<u8>>, // TODO: get rid of refcell?
    error_count: AtomicU64,
}

impl HostUartTransport {
    pub fn new<'a>(
        path: impl Into<Cow<'a, str>>,
        baud_rate: u32,
    ) -> SerialResult<Self> {
        let settings = serialport::new(path, baud_rate)
            .data_bits(DataBits::Eight)
            .flow_control(FlowControl::None)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .timeout(Duration::from_secs(1));

        Self::new_with_config(settings)
    }

    pub fn new_with_config(
        config: SerialPortBuilder,
    ) -> SerialResult<Self> {
        let serial = config.open()?;

        Ok(Self {
            serial: RefCell::new(serial),
            internal_buffer: RefCell::new(Fifo::new_const()),
            error_count: AtomicU64::new(0),
        })
    }

    // Just a helper that increments the error counter:
    fn err<T>(&self, t: T) -> T {
        self.error_count.fetch_add(1, Ordering::Relaxed);
        t
    }
}

// TODO: on std especially we don't need to pass around buffers; we can be
// zero-copy...
impl<const RECV_LEN: usize, SendFormat: super::ConsumeData> Transport<SendFormat, Fifo<u8, RECV_LEN>> for HostUartTransport<RECV_LEN> {
    type RecvErr = Error;
    type SendErr = Error;

    const ID: Identifier =
        Identifier::new_from_str_that_crashes_on_invalid_inputs("UART");
    const VER: Version = {
        let ver = version_from_crate!();

        let id =
            Identifier::new_from_str_that_crashes_on_invalid_inputs("host");

        Version::new(ver.major, ver.minor, ver.patch, Some(id))
    };

    fn send(&self, mut message: SendFormat) -> IoResult<()> {
        let mut serial = self.serial.borrow_mut();

        // TODO: is this still what we want?
        macro_rules! block {
            ($e:expr) => {
                loop {
                    match $e {
                        Ok(()) => break IoResult::Ok(()),
                        Err(e) => match e.kind() {
                            ErrorKind::WouldBlock => continue,
                            _ => return Err(self.err(e)),
                        },
                    }
                }
            };
        }

        // serial.write(message.as_slice()).map(|_| ())?;
        // serial.flush()

        message.consume_slice(|slice| {
            block!(serial.write(slice).map(|_| ()))
        }).unwrap();

        block!(serial.flush())
    }

    fn get(&self) -> Result<Fifo<u8>, Option<Error>> {
        let serial = self.serial.borrow_mut();

        // Ensure that we have bytes before "blocking" and reading in a whole message:
        if serial.bytes_to_read().map_err(|e| self.err(Some(e.into())))? != 0 {
            drop(serial);
            self.blocking_get()
        } else {
            Err(None)
        }
    }

    fn blocking_get(&self) -> Result<Fifo<u8>, Option<Self::RecvErr>> { // TODO: &mut ?
        let mut serial = self.serial.borrow_mut();
        let mut buf = self.internal_buffer.borrow_mut();

        // Note: this is bad! we should accept larger buffers?
        let mut temp_buf = [0; 1];

        // If `get` has been called we expect to produce _something_ (or to timeout).
        loop {
            match serial.read(&mut temp_buf) {
                Ok(1) => {
                    log::trace!("recv byte: {:#04X}", temp_buf[0]);
                    if temp_buf[0] == 0 {
                        log::trace!("message over, returning.. ({} bytes)", buf.len());
                        return Ok(core::mem::replace(&mut buf, Fifo::new()));
                    } else {
                        // TODO: don't panic here; see the note in uart_simple
                        buf.push(temp_buf[0]).unwrap()
                    }
                },
                Ok(0) => {
                    /* shouldn't get here but if we do we'll just try again */
                    log::error!("shouldn't get here??");
                },
                Ok(_) => unreachable!(),
                Err(err) => {
                    log::error!("UART host transport error: {err:?}");

                    return match err.kind() {
                        std::io::ErrorKind::WouldBlock => unreachable!("`serialport` clears O_NONBLOCK..."),
                        std::io::ErrorKind::TimedOut => Err(None),
                        _ => Err(Some(self.err(err))),
                    };
                },
            }
        }
    }

    fn num_get_errors(&self) -> u64 { self.error_count.load(Ordering::Relaxed) }
}
