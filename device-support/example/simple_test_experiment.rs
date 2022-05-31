//Simple host side transport experiment setup. Intended to try to better understand
//nuances with host MCU serial communication and debug reliability issues
// Right now it just experiments with random data with MCU that loops back. Will slowly build up with
// serialized and slip encoded messages and stress test packets and intentional error scenarios with a
//device (TM4C) that runs a similar experimental complete event loop setup and better understand failure scenarios and add
//logging capabilities for RPC

use lc3_device_support::util::Fifo;

use lc3_traits::control::rpc::{Transport, encoding::*};
use lc3_traits::control::{Identifier, Version, version_from_crate};

use serialport::{
    DataBits, FlowControl, Parity, StopBits, SerialPort,
    open_with_settings
};
pub use serialport::SerialPortSettings;

use std::path::Path;
use std::io::{Read, Write, Error, ErrorKind, Result as IoResult};
use std::convert::AsRef;
use std::cell::RefCell;
use std::time::Duration;
use std::ffi::OsStr;

// TODO: Debug impl
pub struct HostUartTransport {
    serial: RefCell<Box<dyn SerialPort>>,
    internal_buffer: RefCell<Fifo<u8>>,
}

impl HostUartTransport {
    pub fn new<P: AsRef<Path>>(path: P, baud_rate: u32) -> IoResult<Self> {
        let settings = SerialPortSettings {
            baud_rate: baud_rate,
            data_bits: DataBits::Eight,
            flow_control: FlowControl::None,
            parity: Parity::None,
            stop_bits: StopBits::One,
            timeout: Duration::from_secs(1),
        };

        Self::new_with_config(path, settings)
    }

    pub fn new_with_config<P: AsRef<Path>>(path: P, config: SerialPortSettings) -> IoResult<Self> {
        let serial = open_with_settings(AsRef::<OsStr>::as_ref(path.as_ref()), &config)?;

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

    const ID: Identifier = Identifier::new_from_str_that_crashes_on_invalid_inputs("UART");
    const VER: Version = {
        let ver = version_from_crate!();

        let id = Identifier::new_from_str_that_crashes_on_invalid_inputs("host");

        Version::new(ver.major, ver.minor, ver.patch, Some(id))
    };

    fn send(&self, message: Fifo<u8>) -> IoResult<()> {
        let mut serial = self.serial.borrow_mut();
        let string = "hello! this is a very long string test. Infact this is even longer";
        let mut serial_buf: Vec<u8> = vec![0; 1000];
                match serial.write(message.as_slice()) {
                    Ok(_) => {
                        //print!("{}", &string);
                        std::io::stdout().flush().unwrap();
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
                    Err(e) => eprintln!("{:?}", e),
                }
        Ok(())
    }

    fn get(&self) -> Result<Fifo<u8>, Option<Error>> {
        std::thread::sleep(Duration::from_millis(100));
        let mut serial_buf: Vec<u8> = vec![0; 1000];
        let mut fifo = Fifo::new();
        match self.serial.borrow_mut().read(serial_buf.as_mut_slice()) {  //TODO: Fix direct try to write to fifo issue.
            Ok(t) => {
                println!("Received some data: num bytes = {}", t);
                for i in 0..t {
                    fifo.push(serial_buf[i]);
                }
                //std::io::stdout().write_all(&serial_buf[..t]).unwrap()
            },
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        }
        
        Ok(fifo)
    }

}














use lc3_traits::control::rpc::controller::*;
use lc3_traits::control::rpc::futures::*;
//use lc3_traits::control::rpc::messages::{RequestMessage, ResponseMessage};
use lc3_traits::control::rpc::*;

use lc3_traits::control::Control;
//use lc3_device_support::rpc::transport::uart_host::*;

    struct unit;

    impl Encode<u32> for unit {
        type Encoded = u32;
        fn encode(&mut self, m: &u32) -> u32 { *m as u32 }
    }

    impl Decode<u32> for unit {
        type Encoded = u32;
        type Err = core::num::TryFromIntError;

        fn decode(&mut self, e: &u32) -> Result<u32, Self::Err> {
            unimplemented!()
           // core::convert::TryInto::try_into(*e)
        }
    }


        struct transport_unit(HostUartTransport);
    
    impl Transport<RequestMessage, ResponseMessage> for transport_unit {
        type RecvErr = u32;
        type SendErr = u32;

        const ID: Identifier =Identifier::new_from_str_that_crashes_on_invalid_inputs("MPSC");
        const VER: Version = version_from_crate!();

        fn send(&self, message: RequestMessage) -> Result<(), Self::SendErr> {
            let mut fifo = Fifo::new();
            for i in 65..75{
                fifo.push(i);
            }
            self.0.send(fifo);
            //println!("sending {:?}", RequestMessage::GetPc);
            Ok(())
        }

        fn get(&self) -> Result<ResponseMessage, Option<Self::RecvErr>> {
            let mut fifo = self.0.get().unwrap();
            //println!("received {:?}", fifo.pop().unwrap());
            std::io::stdout().write_all(&fifo.as_slice()[..fifo.length()]);
            Ok(ResponseMessage::GetPc(34))
        }
    }




fn main(){
    let x = SyncEventFutureSharedState::new();
    let y = Transparent::<RequestMessage>::default();
    let z = Transparent::<ResponseMessage>::default();
    let mut t2 = HostUartTransport::new("/dev/ttyACM0", 115200).unwrap();
    let t = transport_unit(t2);
    let mut host_controller = 
        Controller::<
            transport_unit,
            SyncEventFutureSharedState,
        >::new(y, z, t, &x);
    host_controller.get_pc();
    //println!("Hello");

    //println!("{:?} {:?}", example, decoded);
}



























    //     let mut serial = self.serial.borrow_mut();
    //     let mut buf = self.internal_buffer.borrow_mut();

    //     // Note: this is bad!

    //     let mut temp_buf = [0; 1];

    //     while serial.bytes_to_read().map_err(|e| Some(e.into()))? != 0 {
    //         match serial.read(&mut temp_buf) {
    //             Ok(1) => {
    //                 if temp_buf[0] == 0 {
    //                     return Ok(core::mem::replace(&mut buf, Fifo::new()))
    //                 } else {
    //                     // TODO: don't panic here; see the note in uart_simple
    //                     buf.push(temp_buf[0]).unwrap()
    //                 }
    //             },
    //             Ok(0) => {},
    //             Ok(_) => unreachable!(),
    //             Err(err) => {
    //                 // if let std::io::ErrorKind::Io(kind) = err.kind() {
    //                 //     if let std::io::ErrorKind::WouldBlock = kind {
    //                 //         return Err(None)
    //                 //     } else {
    //                 //         return Err(Some(err))
    //                 //     }
    //                 // } else {
    //                 //     return Err(Some(err))
    //                 // }

    //                 if let std::io::ErrorKind::WouldBlock = err.kind() {
    //                     println!("came here error");
    //                     return Err(None)
    //                 } else {
    //                     println!("came here error");
    //                     return Err(Some(err))
    //                 }
    //             }
    //         }
    //     }
    //     println!("came here error");
    //     Err(None)
    // }




