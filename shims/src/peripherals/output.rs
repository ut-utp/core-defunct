use lc3_traits::peripherals::output::{Output, OutputError};
use std::io::{stdout, Write};

use crate::peripherals::OwnedOrRef;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use std::io::Result as IoResult;

// Eats characters
pub trait Sink {
    fn put_char(&self, c: u8) -> IoResult<usize>;
    fn flush(&self) -> IoResult<()>;
}

impl<W: Write> Sink for Mutex<W> {
    // TODO: update this for `char` when the time comes and be sure not to
    // release the lock until all the bytes in the char have been written.
    fn put_char(&self, c: u8) -> IoResult<usize> {
        self.lock().unwrap().write(&[c])
    }

    fn flush(&self) -> IoResult<()> {
        self.lock().unwrap().flush()
    }
}

impl<S: Sink> Sink for Arc<S> {
    fn put_char(&self, c: u8) -> IoResult<usize> {
        (**self).put_char(c)
    }

    fn flush(&self) -> IoResult<()> {
        (**self).flush()
    }
}

// TODO: cleanup fallout from flags

// #[derive(Clone)] // TODO: Debug
pub struct OutputShim<'out> {
    sink: OwnedOrRef<'out, dyn Sink + Send + Sync + 'out>,
    flag: AtomicBool,
    interrupt_enable_bit: bool,
}

impl Default for OutputShim<'_> {
    fn default() -> Self {
        Self::using(Box::new(Mutex::new(stdout())))
    }
}

impl<'o> OutputShim<'o> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn using(sink: Box<dyn Sink + Send + Sync + 'o>) -> Self {
        Self {
            sink: OwnedOrRef::Owned(sink),
            flag: AtomicBool::new(true),
            interrupt_enable_bit: false,
        }
    }

    pub fn with_ref(sink: &'o (dyn Sink + Send + Sync + 'o)) -> Self {
        Self {
            sink: OwnedOrRef::Ref(sink),
            flag: AtomicBool::new(true),
            interrupt_enable_bit: false,
        }
    }

    pub fn get_inner_ref(&self) -> &(dyn Sink + Send + Sync + 'o) {
        &*self.sink
    }
}

impl<'out> Output for OutputShim<'out> {
    fn interrupt_occurred(&self) -> bool {
        self.current_data_written()
    }

    fn reset_interrupt_flag(&mut self) {
        self.flag.store(false, Ordering::SeqCst);
    }

    fn set_interrupt_enable_bit(&mut self, bit: bool) {
        self.interrupt_enable_bit = bit;
    }

    fn interrupts_enabled(&self) -> bool {
        self.interrupt_enable_bit
    }

    // TODO: handle OutputErrors to somehow report that the write or flush went wrong
    fn write_data(&mut self, c: u8) -> Result<(), OutputError> {
        // if !c.is_ascii() {
        //     return Err(OutputError::NonUnicodeCharacter(c));
        // }
        // ^ TODO!

        self.flag.store(false, Ordering::SeqCst);
        self.sink.put_char(c)?;
        self.sink.flush()?;
        self.flag.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn current_data_written(&self) -> bool {
        // eprintln!("Output Polled for readiness: {:?}", self.flag.unwrap().load(Ordering::SeqCst));

        let val = self.flag.load(Ordering::SeqCst);
        if !val {
            self.flag.store(true, Ordering::SeqCst);
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use lc3_test_infrastructure::assert_eq;

    #[test]
    fn write_one() {
        let vec = Vec::new();
        let mut sink = Mutex::new(vec);
        let mut shim = OutputShim::with_ref(&mut sink);

        // let mut shim = OutputShim { sink: OwnedOrRef::Ref(&mut sink) };
        let ch0 = 'A' as u8;
        let res = shim.write_data(ch0);
        drop(shim);
        assert_eq!(res, Ok(()));
        assert_eq!(sink.lock().unwrap()[0], ch0);
    }

    #[test]
    fn write_multiple() {
        let vec = Vec::new();
        let flag = AtomicBool::new(false);
        let mut sink = Mutex::new(vec);
        let mut shim = OutputShim::with_ref(&mut sink);
        shim.register_interrupt_flag(&flag);

        let ch0 = 'L' as u8;
        let ch1 = 'C' as u8;
        let ch2 = '-' as u8;
        let ch3 = '3' as u8;
        shim.write_data(ch0).unwrap();
        shim.write_data(ch1).unwrap();
        shim.write_data(ch2).unwrap();
        shim.write_data(ch3).unwrap();
        drop(shim);
        assert_eq!(sink.lock().unwrap()[0], ch0);
        assert_eq!(sink.lock().unwrap()[1], ch1);
        assert_eq!(sink.lock().unwrap()[2], ch2);
        assert_eq!(sink.lock().unwrap()[3], ch3);
    }

    #[test]
    #[ignore]
    // Annoyingly, this does not fail.
    fn write_too_much() {
        let mut buf: [u8; 1] = [0];
        let thing = &mut buf.as_mut();
        let sink = Mutex::new(thing);
        let mut shim = OutputShim::with_ref(&sink);

        let ch0 = 'Y' as u8;
        let ch1 = 'P' as u8;
        shim.write_data(ch0).unwrap();
        let res = shim.write_data(ch1);
        assert_eq!(res, Err(OutputError::IoError));
    }
}
