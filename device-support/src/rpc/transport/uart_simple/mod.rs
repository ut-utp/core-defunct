//! TODO!

use crate::{rpc::encoding::DynFifoBorrow, util::fifo::Fifo};

use super::*;

// A bad facsimile of the `Read` trait, kind of.

pub trait ConsumeData {
    fn consume<E, F: FnMut(u8) -> Result<(), E>>(&mut self, func: F) -> Result<(), E>;

    fn consume_slice<E, F: FnMut(&[u8]) -> Result<(), E>>(&mut self, func: F) -> Result<(), E>;
}

#[allow(unstable_name_collisions)]
fn consume_slice<E, F: FnMut(&[u8]) -> Result<(), E>, const LEN: usize>(fifo: &mut Fifo<u8, LEN>, mut func: F) -> Result<(), E> {
    let (a, b) = fifo.as_slices();
    func(a)?;
    func(b)?;

    for _ in 0..(a.len() + b.len()) { fifo.pop().unwrap(); }
    assert!(fifo.is_empty());

    Ok(())
}

impl<const L: usize> ConsumeData for Fifo<u8, L> {
    fn consume<E, F: FnMut(u8) -> Result<(), E>>(&mut self, mut func: F) -> Result<(), E> {
        for i in self {
            func(i)?
        }

        Ok(())
    }

    fn consume_slice<E, F: FnMut(&[u8]) -> Result<(), E>>(&mut self, func: F) -> Result<(), E> {
        consume_slice(self, func)
    }
}

impl<const L: usize> ConsumeData for &mut Fifo<u8, L> {
    fn consume<E, F: FnMut(u8) -> Result<(), E>>(&mut self, mut func: F) -> Result<(), E> {
        for i in self {
            func(i)?
        }

        Ok(())
    }

    fn consume_slice<E, F: FnMut(&[u8]) -> Result<(), E>>(&mut self, func: F) -> Result<(), E> {
        consume_slice(self, func)
    }
}

impl<'f, const L: usize> ConsumeData for DynFifoBorrow<'f, L> {
    fn consume<E, F: FnMut(u8) -> Result<(), E>>(&mut self, mut func: F) -> Result<(), E> {
        for i in &mut *self.0 {
            func(i)?
        }

        Ok(())
    }

    fn consume_slice<E, F: FnMut(&[u8]) -> Result<(), E>>(&mut self, func: F) -> Result<(), E> {
        consume_slice(&mut *self.0, func)
    }
}

// Doesn't actually _consume_ but that's okay.
impl<'a> ConsumeData for &'a [u8] {
    fn consume<E, F: FnMut(u8) -> Result<(), E>>(&mut self, mut func: F) -> Result<(), E> {
        for i in self.iter() { func(*i)? }

        Ok(())
    }

    fn consume_slice<E, F: FnMut(&[u8]) -> Result<(), E>>(&mut self, mut func: F) -> Result<(), E> {
        func(self)
    }
}

using_std! {
    impl ConsumeData for std::vec::Vec<u8> {
        fn consume<E, F: FnMut(u8) -> Result<(), E>>(&mut self, mut func: F) -> Result<(), E> {
            for i in self.drain(..) { func(i)? }

            Ok(())
        }

        fn consume_slice<E, F: FnMut(&[u8]) -> Result<(), E>>(&mut self, mut func: F) -> Result<(), E> {
            func(self)?;
            /* potentially inconsistent! */
            self.clear();

            Ok(())
        }
    }

    impl ConsumeData for std::collections::VecDeque<u8> {
        fn consume<E, F: FnMut(u8) -> Result<(), E>>(&mut self, mut func: F) -> Result<(), E> {
            for i in self.drain(..) { func(i)? }

            Ok(())
        }

        fn consume_slice<E, F: FnMut(&[u8]) -> Result<(), E>>(&mut self, mut func: F) -> Result<(), E> {
            let (a, b) = self.as_slices();
            /* not atomic! */
            func(a)?;
            func(b)?;
            self.clear();

            Ok(())
        }
    }
}

pub mod device;

using_std! {
    #[cfg_attr(all(docs, not(doctest)), doc(cfg(all(feature = "host_transport", not(target_arch = "wasm32")))))]
    #[cfg(all(feature = "host_transport", not(target_arch = "wasm32")))]
    pub mod host;
}
