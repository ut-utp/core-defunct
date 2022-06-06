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

use lc3_device_support::rpc::encoding::{PostcardEncode, PostcardDecode, Cobs};

// TODO: Debug impl
pub struct HostUartTransportAlternate {
    serial: RefCell<Box<dyn SerialPort>>,
    internal_buffer: RefCell<Fifo<u8>>,
}

impl HostUartTransportAlternate {
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
impl Transport<Fifo<u8>, Fifo<u8>> for HostUartTransportAlternate {
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
            // let string = "hello! this is a very long string test. Infact this is even longer\0";
            // let string_small = "small string\0";
            // let mut long_fifo = Fifo::new();
            // for i in 1..250 {
            //     long_fifo.push(65);
            // }
            //long_fifo.push(0);
            let mut serial_buf: Vec<u8> = vec![0; 1000];
                    match serial.write(message.as_slice()) {
                    Ok(_) => {
                        print!("sending {:?} num bytes = {:?}", message, message.length());
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
                println!("{:?}", fifo);
                std::io::stdout().write_all(&serial_buf[..t]).unwrap();
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


        struct transport_unit(HostUartTransportAlternate);
    
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


type Cont<'ss, EncFunc: FnMut() -> Cobs<Fifo<u8>>> = Controller<
    'ss,
    HostUartTransportAlternate,
    SyncEventFutureSharedState,
    RequestMessage,
    ResponseMessage,
    PostcardEncode<RequestMessage, Cobs<Fifo<u8>>, EncFunc>,
    PostcardDecode<ResponseMessage, Cobs<Fifo<u8>>>,
>;

use lc3_isa::{Addr, Word};
use std::convert::TryInto;
fn main(){
    let x = SyncEventFutureSharedState::new();    
    let y = Transparent::<RequestMessage>::default();
    let z = Transparent::<ResponseMessage>::default();


    let mut t2 = HostUartTransportAlternate::new("/dev/ttyACM0", 1500000).unwrap();
    //let t = transport_unit(t2);

        let func: Box<dyn FnMut() -> Cobs<Fifo<u8>>> = Box::new(|| Cobs::try_new(Fifo::new()).unwrap());

        let mut controller: Cont<Box<dyn FnMut() -> Cobs<Fifo<u8>>>>  = Controller::new(
            PostcardEncode::new(func),
            PostcardDecode::new(),
            t2,
            &x
        );

    // let mut host_controller = 
    //     Controller::<
    //         PostcardEncode::<ResponseMessage, _, _>,
    //         PostcardDecode::<RequesteMessage, Cobs<Fifo<u8>>>,
    //         transport_unit,
    //         SyncEventFutureSharedState,
    //     >::new(enc, dec, t, &x);
    //controller.transport.send(controller.enc.borrow_mut().encode(RequestMessage::GetPc.into())).unwrap();
    //controller.transport.get();
    controller.set_pc(1004);
    let mut pc = controller.get_pc();
    assert_eq!(pc, 1004);


    use lc3_isa::{Reg};
    controller.set_register(Reg::R0, 42);
    let mut r0 = controller.get_register(Reg::R0);
    assert_eq!(r0, 42);
























    // let get_registers_psr_and_pc = controller.get_registers_psr_and_pc();

     use lc3_traits::control::load::*;

    // //controller.start_page_write(LoadApiSession<PageWriteStart>::new(0), hash_page());
    // //controller.send_page_chunk(&mut self, offset: LoadApiSession<Offset>, chunk: [Word; CHUNK_SIZE_IN_WORDS as usize]);
    // //controller.finish_page_write(&mut self, page: LoadApiSession<PageIndex>);

    // for i in 0..1000 {
    //     controller.write_word(i, i as u16);
    //     let addr0 = controller.read_word(i);
    //     println!("Addr {:?} = {:?}", i, addr0);
    //     assert_eq!(i, addr0);
    // }

    //     // macro_rules! p {
    //     //     ($p:ident -> $($all:tt)*) => { if let Some($p) = progress { $($all)* }};
    //     // }


        let data: [Word; 256] = [4; 256];

        let page = &data;
        let p_idx = 0;
        let checksum = hash_page(page); // We'll use a hash of the page as our checksum for now.

        loop {
            // Start the page write:
            let token = /*loop*/ {
                // (this is safe; see the blurb at the top of this function)
                #[allow(unsafe_code)]
                let page = unsafe { LoadApiSession::new(p_idx as PageIndex) }.unwrap();

               // p!(p -> p.page_attempt());
                match controller.start_page_write(page, checksum) {
                    Ok(token) => token,
                    Err(StartPageWriteError::InvalidPage { .. }) => unreachable!(),
                    Err(StartPageWriteError::UnfinishedSessionExists { unfinished_page }) => {
                        // Bail:
                        panic!()
                    }
                }
            };

            let mut non_empty_pages = 0;

            // Now try to go write all the (non-empty) pages:
            for (idx, chunk) in page.chunks_exact(CHUNK_SIZE_IN_WORDS as usize).enumerate() {
                if chunk.iter().any(|w| *w != 0) {
                    non_empty_pages += 1;

                    let offset = token.with_offset(Index(p_idx as PageIndex).with_offset(idx as PageOffset * CHUNK_SIZE_IN_WORDS)).unwrap();
                    let chunk = chunk.try_into().unwrap();

                //    p!(p -> p.chunk_attempt());
                    match controller.send_page_chunk(offset, chunk) {
                        Ok(()) => { },
                        Err(PageChunkError::ChunkCrossesPageBoundary { .. }) |
                        Err(PageChunkError::NoCurrentSession) |
                        Err(PageChunkError::WrongPage { .. }) => unreachable!(),
                    }
                }
            }

            // Finally, finish the page:
            match controller.finish_page_write(token) {
                Ok(()) => {  break; }
                Err(FinishPageWriteError::NoCurrentSession) |
                Err(FinishPageWriteError::SessionMismatch { .. }) => unreachable!(),
                Err(FinishPageWriteError::ChecksumMismatch { page, given_checksum, computed_checksum }) => {
                    assert_eq!(page, p_idx as u8);
                    assert_eq!(checksum, given_checksum);
                    assert_ne!(checksum, computed_checksum);

                    // We'll try again...
                }
            }
        }


        let addrj = controller.read_word(0);
        assert_eq!(4, addrj);

























    println!("PC = {:?}", pc);
    println!("R0 = {:?}", r0);
    

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




