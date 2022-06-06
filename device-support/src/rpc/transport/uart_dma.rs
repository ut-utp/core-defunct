//TODO: Revisit trying to use bbqueue buffers here. Right now with the existing RPC design there isn't any sensible way
// to use bbbuffers. Problem is bbqueue producesa producer, consumer pair and the producer has to "commit" a certain
// number of bytes before the consumer can read it. And it gets consumed upon commiting. So consuming is a 1 time
// only operation. But we don't really know when to commit since the messages are variable lenghth and we don't 
// know when the sentinel is hit either. If the messages were fixed length or if messages were variable but
// always an integral multiple of some constant, we could use a producer consumer pair for each "chunk". Also,
// if the device had some way of telling when it got a specific byte (sentinel), we know we are done and can commit the data. 
// But right now, I see no viable way to use bbqueue. One slightly gross way to do it might be to just loop through the bbqueue raw
// internal buffer (.buf().as_ptr()) and search for the sentinel within the first dma_num_bytes_transferred() of the buffer and
// commit read and consume if the sentinel is found. 
// For now though just using the Fifo internal buffer.
// TODO: Error checking and buffer limit overflows in dma operation

use crate::util::Fifo;

use lc3_traits::control::rpc::Transport;
use lc3_traits::control::{Identifier, Version, version_from_crate};

use embedded_hal::serial::{Read, Write};
use nb::block;

use core::cell::RefCell;
use core::fmt::Debug;

use bbqueue::{BBBuffer, GrantR, GrantW, Consumer, Producer};


//A trait for  a dma channel. A physical peripheral
pub trait DmaChannel {

    //Device secific preinitialization to enable DMA
    fn dma_device_init(&mut self);

    /// Data will be written to this `address`
    ///
    /// `inc` indicates whether the address will be incremented after every byte transfer
    ///
    /// NOTE this performs a volatile write
    fn dma_set_destination_address(&mut self, address: usize);

    /// Data will be read from this `address`
    ///
    /// `inc` indicates whether the address will be incremented after every byte transfer
    ///
    /// NOTE this performs a volatile write
    fn dma_set_source_address(&mut self, address: usize);

    /// Number of bytes to transfer
    ///
    /// NOTE this performs a volatile write
    fn dma_set_transfer_length(&mut self, len: usize);

    /// Starts the DMA transfer
    ///
    /// NOTE this performs a volatile write
    fn dma_start(&mut self);

    /// Stops the DMA transfer
    ///
    /// NOTE this performs a volatile write
    fn dma_stop(&mut self);

    /// Returns `true` if there's a transfer in progress
    ///
    /// NOTE this performs a volatile read
    fn dma_in_progress(&mut self) -> bool;

    fn dma_num_bytes_transferred(&mut self) -> usize;

}

//static BUF: BBBuffer<256> = BBBuffer::new();

static mut buf: [u8; 1024] = [1; 1024];


//#[derive(Clone)]
pub struct BBBuffer_elements
{
    dma_buffer_prod: Producer<'static, 256>,
    dma_buffer_cons: Consumer<'static, 256>,    
}

// impl BBBuffer_elements{
//     fn new()->Self{
//         //let dma_buffer: BBBuffer<256> = BBBuffer::new();
//         let (prod, cons) = BUF.try_split().unwrap();
//         Self{
//             dma_buffer_prod: prod,
//             dma_buffer_cons: cons
//         }
//     }
// }

//#[derive(Debug)]
pub struct UartDmaTransport<T, R: Read<u8>, W: Write<u8>>
where
    T: DmaChannel,
    <R as Read<u8>>::Error: Debug,
    <W as Write<u8>>::Error: Debug,
{
    dma_unit: RefCell<T>,
    //dma_buffer: ,
   // bbbuffer: RefCell<BBBuffer_elements>,
   // bbbuffer_grant: RefCell<Option<GrantW<'static, 256>>>,
    read: RefCell<R>,
    write: RefCell<W>,
    internal_buffer: RefCell<Fifo<u8>>,
}

impl<T, R: Read<u8>, W: Write<u8>> UartDmaTransport<T, R, W>
where
    <R as Read<u8>>::Error: Debug,
    <W as Write<u8>>::Error: Debug,
    T: DmaChannel,
{
    // Can't be const until bounds are allowe= d.
    pub /*const*/ fn new(read: R, write: W, dma_unit: T) -> Self {
        //let dma_buffer: BBBuffer<256> = BBBuffer::new();
        //let (prod, cons) = dma_buffer.try_split().unwrap();
        Self {
            dma_unit: RefCell::new(dma_unit),
            //dma_buffer: BBBuffer::new(),
            //bbbuffer: RefCell::new(BBBuffer_elements::new()),
            //bbbuffer_grant: RefCell::new(None),
            read: RefCell::new(read),
            write: RefCell::new(write),
            internal_buffer: RefCell::new(Fifo::new_const()),
        }
    }
}


impl<T, W: Write<u8>, R: Read<u8>> Transport<Fifo<u8>, Fifo<u8>> for UartDmaTransport<T, R, W>
where
    T: DmaChannel,
    <R as Read<u8>>::Error: Debug,
    <W as Write<u8>>::Error: Debug,
{
    type RecvErr = u32;
    type SendErr = W::Error;

    const ID: Identifier = Identifier::new_from_str_that_crashes_on_invalid_inputs("UART");
    const VER: Version = {
        let ver = version_from_crate!();

        let id = Identifier::new_from_str_that_crashes_on_invalid_inputs("simp");

        Version::new(ver.major, ver.minor, ver.patch, Some(id))
    };

    fn send(&self, message: Fifo<u8>) -> Result<(), W::Error> {
        let mut write = self.write.borrow_mut();

        for byte in message {
            write.write(byte).unwrap();
        }
        block!(write.flush())
    }

    fn get(&self) -> Result<Fifo<u8>, Option<u32>> {

        let mut dma_unit = self.dma_unit.borrow_mut();
        //let mut bbbuffer = self.bbbuffer.borrow_mut();
        //let mut bbbuffer_grant = self.bbbuffer_grant.borrow_mut();
        let mut internal_buffer = self.internal_buffer.borrow_mut();

        // if(!dma_unit.dma_in_progress()){
        // // Request space for max message size
        //     //*bbbuffer_grant = Some(bbbuffer.dma_buffer_prod.grant_exact(256).unwrap());
        //     let mut addr: GrantW<256> = bbbuffer.dma_buffer_prod.grant_exact(256).unwrap();
        //     //core::mem::swap(&mut *bbbuffer_grant, &mut addr);
        //     dma_unit.dma_device_init();
        //     dma_unit.dma_set_destination_address(addr.buf().as_ptr() as usize);
        //     dma_unit.dma_set_transfer_length(256);
        //     *bbbuffer_grant = Some(addr);
        //     dma_unit.dma_start();
            
        //     //core::mem::swap(&mut addr, &mut *bbbuffer_grant);
        // }

        // else{
        //     //let mut addr: Option<GrantW<256>> = (bbbuffer_grant);
        //     let mut addr: Option<GrantW<256>> = None;
        //     let bytes_transferred = dma_unit.dma_num_bytes_transferred();

        //     if(bytes_transferred > 0){
        //         core::mem::swap(&mut *bbbuffer_grant, &mut addr);
        //         let mut temp: GrantW<256> = addr.unwrap();
            
        //        // temp.commit(bytes_transferred);
        //         let data = bbbuffer.dma_buffer_cons.read().unwrap();
        //         for i in 0..bytes_transferred{
        //             internal_buffer.push(data[i]).unwrap()
        //         }

        //         data.release(bytes_transferred);
        //         *bbbuffer_grant = Some(temp);

        //     }


            
        //     if( bytes_transferred == 0){

        //     }

        // }

















         let mut sentinel_found = false;
         let mut message_size = 0;

        if(!dma_unit.dma_in_progress()){  //should probably be done in main initialization? seems like a one time only operation since the progress
                                          // always returns true as long as not all 256 max bytes are filled and no message is typically that long.
            dma_unit.dma_device_init();
            //dma_unit.dma_set_destination_address(internal_buffer.as_ref().as_ptr() as *const u8 as usize); This doesn't work for some reason
            unsafe{dma_unit.dma_set_destination_address(&buf as *const u8 as usize);};
            dma_unit.dma_set_transfer_length(1024);
            dma_unit.dma_start();
            
        }
        else{
            let bytes_transferred = dma_unit.dma_num_bytes_transferred();
            for i in 0..bytes_transferred{
                unsafe{
                    if(buf[i] == 0){
                        message_size = bytes_transferred;
                        sentinel_found = true;
                        dma_unit.dma_stop();

                        for j in 0..(bytes_transferred-1){
                            internal_buffer.push(buf[j]).unwrap();
                        }
                        // reset and start next cycle
                        
                        dma_unit.dma_device_init(); 
                        //dma_unit.dma_set_destination_address(internal_buffer.as_ref().as_ptr() as *const u8 as usize);
                        unsafe{dma_unit.dma_set_destination_address(&buf as *const u8 as usize);};
                        dma_unit.dma_set_transfer_length(1024);
                        dma_unit.dma_start();
                    }
                };
            }
        }
        let mut ret: Result<Fifo<u8>, Option<u32>> = Err(None);
        if(sentinel_found){
            unsafe{
            let one = buf[0];
            let two = buf[1];
            let three = buf[2];
            let four = buf[3];
            let five = buf[4];
            let six = buf[5];
            let seven = buf[6];
        }

            ret = Ok(core::mem::replace(&mut internal_buffer, Fifo::new()));
        }

         ret
    }
}
