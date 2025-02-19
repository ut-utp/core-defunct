//! UART transport for computers.

use crate::util::Fifo;

use lc3_traits::control::rpc::Transport;
use lc3_traits::control::{version_from_crate, Identifier, Version};

use serialport::{
    DataBits, FlowControl, Parity, Result as SerialResult, SerialPort,
    StopBits,
};
pub use serialport::SerialPortBuilder;

use std::borrow::Cow;
use std::cell::RefCell;
use std::io::{Error, ErrorKind, Read, Result as IoResult, Write};
use std::time::Duration;

// TODO: Debug impl
pub struct HostUartTransport {
    serial: RefCell<Box<dyn SerialPort>>,
    internal_buffer: RefCell<Fifo<u8>>,
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
            .timeout(Duration::from_secs(10));

        Self::new_with_config(settings)
    }

    pub fn new_with_config(
        config: SerialPortBuilder,
    ) -> SerialResult<Self> {
        let serial = config.open()?;

        Ok(Self {
            serial: RefCell::new(serial),
            internal_buffer: RefCell::new(Fifo::new_const()),
        })
    }
}

// TODO: on std especially we don't need to pass around buffers; we can be
// zero-copy...
impl Transport<Fifo<u8>, Fifo<u8>> for HostUartTransport {
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

    fn send(&self, message: Fifo<u8>) -> IoResult<()> {
        let mut serial = self.serial.borrow_mut();

        // TODO: is this still what we want?
        macro_rules! block {
            ($e:expr) => {
                loop {
                    match $e {
                        Ok(()) => break IoResult::Ok(()),
                        Err(e) => match e.kind() {
                            ErrorKind::WouldBlock => continue,
                            _ => return Err(e),
                        },
                    }
                }
            };
        }

        // serial.write(message.as_slice()).map(|_| ())?;
        // serial.flush()

        block!(serial.write(message.as_slice()).map(|_| ())).unwrap();
        block!(serial.flush())
    }

    fn get(&self) -> Result<Fifo<u8>, Option<Error>> {
        let mut serial = self.serial.borrow_mut();
        let mut buf = self.internal_buffer.borrow_mut();

        // Note: this is bad!

        let mut temp_buf = [0; 1];

        while serial.bytes_to_read().map_err(|e| Some(e.into()))? != 0 {
            match serial.read(&mut temp_buf) {
                Ok(1) => {
                    if temp_buf[0] == 0 {
                        return Ok(core::mem::replace(&mut buf, Fifo::new()));
                    } else {
                        // TODO: don't panic here; see the note in uart_simple
                        buf.push(temp_buf[0]).unwrap()
                    }
                }
                Ok(0) => {}
                Ok(_) => unreachable!(),
                Err(err) => {
                    // if let std::io::ErrorKind::Io(kind) = err.kind() {
                    //     if let std::io::ErrorKind::WouldBlock = kind {
                    //         return Err(None)
                    //     } else {
                    //         return Err(Some(err))
                    //     }
                    // } else {
                    //     return Err(Some(err))
                    // }

                    if let std::io::ErrorKind::WouldBlock = err.kind() {
                        return Err(None);
                    } else {
                        return Err(Some(err));
                    }
                }
            }
        }

        Err(None)
    }
}
