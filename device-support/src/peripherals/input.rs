use lc3_traits::peripherals::input::*;
use lc3_traits::control::rpc::device::DeviceKeyboard;
// extern crate embedded_hal;
// use embedded_hal as hal;
// use embedded_hal::adc::{Channel, OneShot};
use core::marker::PhantomData;
use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, Ordering};

//Since Keyboard is not a real device on a microcontroller and the keyboard data has to come from host, simulating
//this as a virtual peripheral. The actual keyboard data comes from host via rpc.
pub struct VirtualKeyboard<'a>{
   keyboard: &'a mut DeviceKeyboard,
   interrupt_flag: Option<&'a AtomicBool>,
   interrupt_bit: AtomicBool,
}

impl <'a> Default for VirtualKeyboard<'a>{
    fn default() -> Self {
        unimplemented!()
    }
}

impl <'a> VirtualKeyboard<'a>{
    fn new(keyboard: &'a mut DeviceKeyboard) -> Self {
        Self{
            keyboard: keyboard,
            interrupt_flag: None,
            interrupt_bit: AtomicBool::new(false),
        }
    }
}

impl<'a> Input<'a> for VirtualKeyboard<'a> {
//     // Warning! This is stateful!! It marks the current data as read.
//     //
//     // Also note: this is technically infallible (it's up to the
//     // interpreter what to do for some of the edge cases, but
//     // we'll presumably just return some default value) but since
//     // we're letting the interpreter decide we *do* return a Result
//     // type here.
//     //
//     // Must use interior mutability.
     fn read_data(&self) -> Result<u8, InputError> {
        let mut ret = Err(InputError::NoDataAvailable);
        match self.keyboard.read_data(){
            Some(data) => {
                ret = Ok(data)
            }
            _ => {}
        }
        ret
     }
     fn current_data_unread(&self) -> bool {
        self.keyboard.new_data_ready()
     }

     fn register_interrupt_flag(&mut self, flag: &'a AtomicBool) {
        self.interrupt_flag = Some(flag);
     }
     fn interrupt_occurred(&self) -> bool {
        self.keyboard.new_data_ready() && self.interrupts_enabled()
     }
     fn reset_interrupt_flag(&mut self) {
        self.interrupt_flag.unwrap().store(false, Ordering::SeqCst);
     }

     fn set_interrupt_enable_bit(&mut self, bit: bool) {
        self.interrupt_bit.store(bit, Ordering::SeqCst);
     }
     fn interrupts_enabled(&self) -> bool {
        self.interrupt_bit.load(Ordering::SeqCst)
     }
 }