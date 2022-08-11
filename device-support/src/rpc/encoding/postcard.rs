//! TODO!

use crate::util::Fifo;

use lc3_traits::control::rpc::{Encode, Decode};

use serde::{Serialize, Deserialize};
use postcard::ser_flavors::{Flavor as SerFlavor, Cobs};
use postcard::serialize_with_flavor;
use postcard::take_from_bytes_cobs;

use core::cell::RefMut;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::ops::{IndexMut, Index};
use core::convert::{AsRef, AsMut};

// TODO: have be able to take inputs?
// no: they're closures; just capture.

mod encode {
    use super::*;

    /* trait GiveFlavor {
        type Output<'a>: SerFlavor;

        fn give<'a>(&'a mut self) -> Self::Output<'a>;
    }
    */
    // pub trait GiveFlavorLifetime<'this, Implicit = &'this Self> {
    //     type Output;
    // }
    // pub trait GiveFlavor
    // where
    //     Self: for<'this> GiveFlavorLifetime<'this, &'this Self>,
    // {
    //     fn give<'a>(&'a mut self) -> <Self as GiveFlavorLifetime<'a>>::Output;
    // }

    // impl<'f, O, F: FnMut() -> O> GiveFlavorLifetime<'f> for F {
    //     type Output = O;
    // }
    // impl<O, F: FnMut() -> O> GiveFlavor for F {
    //     fn give(&mut self) -> O { (self)() }
    // }

    // struct FifoBorrow<'f, const L: usize>(&'f mut Fifo<u8, L>);
    // impl<'this, 'f, const L: usize> GiveFlavorLifetime<'this, &'this Self> for FifoBorrow<'f, L>
    // where
    // {
    //     type Output = Cobs<&'this mut Fifo<u8, L>>;
    // }
    // impl<'f, const L: usize> GiveFlavor for FifoBorrow<'f, L> {
    //     fn give(&mut self) -> <Self as GiveFlavorLifetime<'_>>::Output {
    //         Cobs::try_new(self.0).unwrap()
    //     }
    // }

    // #[derive(Debug, Default)]
    // pub struct PostcardEncode<'f, Inp: ?Sized, Func>
    // // where
    // //     Inp: ?Sized + Debug + Serialize,
    // //     F: SerFlavor,
    // //     <F as SerFlavor>::Output: Debug,
    // //     Func: FnMut() -> F,
    // {
    //     flavor_ctor_func: Func,
    //     _i: PhantomData<(&'f (), Inp)>,
    // }

    // impl<'f, Inp, Func> PostcardEncode<'f, Inp, Func>
    // where
    //     Inp: ?Sized + Debug + Serialize,
    //     Func: GiveFlavor,
    //     for<'a> <Func as GiveFlavorLifetime<'a>>::Output: SerFlavor,
    //     for<'a> <<Func as GiveFlavorLifetime<'a>>::Output as SerFlavor>::Output: Debug,
    // {
    //     pub const fn new(flavor_ctor_func: Func) -> Self {
    //         Self {
    //             flavor_ctor_func,
    //             _i: PhantomData,
    //         }
    //     }
    // }

    // impl PostcardEncode<'_, (), ()> {
    //     pub /*const*/ fn with_cobs<'f, Inp, FlavGiv>(mut inner_flavor_func: FlavGiv) -> PostcardEncode<'f, Inp, impl GiveFlavor>
    //     where
    //         Inp: ?Sized + Debug + Serialize,
    //         FlavGiv: GiveFlavor,
    //         for<'a> <FlavGiv as GiveFlavorLifetime<'a>>::Output: SerFlavor,
    //         for<'a> <FlavGiv as GiveFlavorLifetime<'a>>::Output: IndexMut<usize, Output = u8>,
    //         for<'a> <<FlavGiv as GiveFlavorLifetime<'a>>::Output as SerFlavor>::Output: Debug,
    //     {
    //         // Ok(PostcardEncode::new(Cobs::try_new(inner_flavor)?))

    //         PostcardEncode::new(move || Cobs::try_new(inner_flavor_func.give()).unwrap())
    //     }
    // }

    // // TODO: remove `I` here; it's too strict!
    // impl PostcardEncode<'_, (), ()> {
    //     pub /*const*/ fn with_cobs<'a, Inp, I, Func>(mut inner_flavor_func: Func) -> PostcardEncode<'a, Inp, impl FnMut() -> Cobs<I>>
    //     where
    //         Inp: ?Sized + Debug + Serialize,
    //         I: SerFlavor,
    //         I: IndexMut<usize, Output = u8>,
    //         <I as SerFlavor>::Output: Debug,
    //         Func: GiveFlavor<Output = I>,
    //     {
    //         // Ok(PostcardEncode::new(Cobs::try_new(inner_flavor)?))

    //         PostcardEncode::new(move || Cobs::try_new(inner_flavor_func.give()).unwrap())
    //     }
    // }

    // impl<'a, Inp> PostcardEncode<Inp, Cobs<Slice<'a>>>
    // where
    //     Inp: ?Sized + Debug + Serialize,
    // {
    //     pub /*const*/ fn with_slice(buffer: &'a mut [u8]) -> postcard::Result<Self> {
    //         Ok(PostcardEncode::new(Cobs::try_new(Slice::new(buffer))?))
    //     }
    // }

    // pub type PostcardFifoCobs<Inp> = PostcardEncode<Inp, Cobs<Fifo<u8>>>;

    // impl PostcardEncode<'_, (), ()> {
        // pub /*const*/ fn with_cobs_fifo_func<'fifo, Inp, Func, const L: usize>(
        //     fifo: &'fifo mut Fifo<u8, L>,
        // ) -> PostcardEncode<
        //     Inp,
        //     Cobs<&'fifo mut Fifo<u8, L>>,
        //     impl FnMut() -> Cobs<&'fifo mut Fifo<u8, L>> + 'fifo,
        // >
        // where
        //     Inp: ?Sized + Debug + Serialize + 'fifo,
        //     Func: 'fifo,
        //     Func: FnMut() -> &'fifo mut Fifo<u8, L>,
        // {
        //     let borrow = FifoBorrow(fifo);
        //     PostcardEncode::<Inp, _, _>::with_cobs(borrow)
        // }

        // pub /*const*/ fn with_cobs_fifo<Inp, const L: usize>(
        //     fifo: &mut Fifo<u8, L>,
        // ) -> PostcardEncode<
        //     Inp,
        //     FifoBorrow<'_, L>,
        // >
        // where
        //     Inp: ?Sized + Debug + Serialize,
        // {
        //     let borrow = FifoBorrow(fifo);
        //     PostcardEncode::<Inp, _>::new(borrow)
        // }

        // // The invariance of lifetimes behind mutable references keeps us from
        // // offerring the API we want to here; i.e. having a function yield
        // // mutable references to underlying Fifo that outlives the function.
        // //
        // // The issue is that if we shorten the lifetime of the `&mut Fifo` that
        // // we yield, someone can replace the underlying instance with

        // pub /*const*/ fn with_cobs_fifo_func<'fifo, Inp, Func, const L: usize>(
        //     fifo_func: Func,
        // ) -> PostcardEncode<
        //     Inp,
        //     Cobs<&'fifo mut Fifo<u8, L>>,
        //     impl FnMut() -> Cobs<&'fifo mut Fifo<u8, L>> + 'fifo,
        // >
        // where
        //     Inp: ?Sized + Debug + Serialize + 'fifo,
        //     Func: 'fifo,
        //     Func: FnMut() -> &'fifo mut Fifo<u8, L>,
        // {

        //     PostcardEncode::with_cobs::<Inp, _, _>(fifo_func)
        // }
    // }

    // impl<'f, Inp, Func> Encode<Inp> for PostcardEncode<'f, Inp, Func>
    // where
    //     Inp: Debug + Serialize,
    //     Func: GiveFlavor + 'f,
    //     for<'a> <Func as GiveFlavorLifetime<'a>>::Output: SerFlavor,
    //     for<'a> <<Func as GiveFlavorLifetime<'a>>::Output as SerFlavor>::Output: Debug,
    // {
    //     type Encoded = <<Func as GiveFlavorLifetime<'f>>::Output as SerFlavor>::Output;

    //     fn encode(&mut self, message: &Inp) -> Self::Encoded {
    //         serialize_with_flavor(message, self.flavor_ctor_func.give())
    //             .expect("a successful encode")
    //     }
    // }

    #[derive(Debug, Default)]
    pub struct PostcardEncode<Inp, F, Func>
    where
        Inp: ?Sized + Debug + Serialize,
        F: SerFlavor,
        <F as SerFlavor>::Output: Debug,
        Func: FnMut() -> F,
    {
        // flavor: F,
        flavor_func: Func,
        _i: PhantomData<Inp>,
    }

    impl<Inp, F, Func> PostcardEncode<Inp, F, Func>
    where
        Inp: ?Sized + Debug + Serialize,
        F: SerFlavor,
        <F as SerFlavor>::Output: Debug,
        Func: FnMut() -> F,
    {
        // Once we can have const fns with real trait bounds this can be const.
        // pub /*const*/ fn new(flavor: F) -> Self {
        //     Self {
        //         flavor,
        //         _i: PhantomData,
        //     }
        // }

        pub fn new(flavor_func: Func) -> Self {
            Self {
                flavor_func,
                _i: PhantomData
            }
        }
    }

    impl<Inp, I, CFunc> PostcardEncode<Inp, Cobs<I>, CFunc>
    where
        Inp: ?Sized + Debug + Serialize,
        I: SerFlavor,
        I: IndexMut<usize, Output = u8>,
        <I as SerFlavor>::Output: Debug,
        CFunc: FnMut() -> Cobs<I>,
    {
        pub /*const*/ fn with_cobs(mut inner_flavor_func: impl FnMut() -> I) -> PostcardEncode<Inp, Cobs<I>, impl FnMut() -> Cobs<I>> {
            // Ok(PostcardEncode::new(Cobs::try_new(inner_flavor)?))

            PostcardEncode::new(move || Cobs::try_new((inner_flavor_func)()).unwrap())
        }
    }

    // impl<'a, Inp> PostcardEncode<Inp, Cobs<Slice<'a>>>
    // where
    //     Inp: ?Sized + Debug + Serialize,
    // {
    //     pub /*const*/ fn with_slice(buffer: &'a mut [u8]) -> postcard::Result<Self> {
    //         Ok(PostcardEncode::new(Cobs::try_new(Slice::new(buffer))?))
    //     }
    // }

    // pub type PostcardFifoCobs<Inp> = PostcardEncode<Inp, Cobs<Fifo<u8>>>;

    impl<Inp, Func> PostcardEncode<Inp, Cobs<Fifo<u8>>, Func>
    where
        Inp: ?Sized + Debug + Serialize,
        Func: FnMut() -> Cobs<Fifo<u8>>,
    {
        // pub /*const*/ fn with_fifo() -> postcard::Result<Self> {
            // Ok(PostcardEncode::new(Cobs::try_new(Fifo::new())?))
        // pub /*const*/ fn with_fifo() -> postcard::Result<Self> {
        pub /*const*/ fn with_fifo() -> PostcardEncode<Inp, Cobs<Fifo<u8>>, impl FnMut() -> Cobs<Fifo<u8>>> {

            PostcardEncode::<Inp, _, Func>::with_cobs(|| Fifo::new())
        }
    }

    impl<Inp, F, Func> Encode<Inp> for PostcardEncode<Inp, F, Func>
    where
        Inp: Debug + Serialize,
        F: SerFlavor,
        <F as SerFlavor>::Output: Debug,
        Func: FnMut() -> F,
    {
        type Encoded = <F as SerFlavor>::Output;

        fn encode(&mut self, message: &Inp) -> <F as SerFlavor>::Output {
            serialize_with_flavor(message, (self.flavor_func)())
                .expect("a successful encode")
        }
    }
}

mod decode {
    use super::*;

    // TODO: have a default like this for PostcardEncode (Cobs<Fifo<u8>>)
    #[derive(Debug, Default)]
    pub struct PostcardDecode<Out, F = Cobs<Fifo<u8>>>
    where
        Out: Debug,
        for<'de> Out: Deserialize<'de>,
        F: SerFlavor,
        <F as SerFlavor>::Output: Debug,
    {
        _f: PhantomData<F>,
        _o: PhantomData<Out>,
    }

    impl<Out, F> PostcardDecode<Out, F>
    where
        Out: Debug,
        for<'de> Out: Deserialize<'de>,
        F: SerFlavor,
        <F as SerFlavor>::Output: Debug,
    {
        pub /*const*/ fn new() -> Self {
            Self {
                _f: PhantomData,
                _o: PhantomData,
            }
        }
    }

    // We can't provide full generality because there's no DeFlavor trait.
    // Unclear whether we can to better than the below (Cobs + AsMut). TODO.

    impl<F, Out> Decode<Out> for PostcardDecode<Out, Cobs<F>>
    where
        Out: Debug,
        for<'de> Out: Deserialize<'de>,
        F: SerFlavor,
        F: IndexMut<usize, Output = u8>,
        Cobs<F>: SerFlavor,
        <Cobs<F> as SerFlavor>::Output: Debug,
        <Cobs<F> as SerFlavor>::Output: AsMut<[u8]>
    {
        type Encoded = <Cobs<F> as SerFlavor>::Output;
        type Err = postcard::Error;

        fn decode(&mut self, encoded: Self::Encoded) -> Result<Out, Self::Err> {
            take_from_bytes_cobs(encoded.as_mut())
                .map(|(m, _)| m)
        }
    }
}

impl<const L: usize> SerFlavor for Fifo<u8, L> {
    type Output = Self;

    fn try_push(&mut self, data: u8) -> postcard::Result<()> {
        self.push(data).map_err(|()| postcard::Error::SerializeBufferFull)
    }

    fn finalize(self) -> postcard::Result<Self::Output> {
        Ok(self)
    }

    fn try_extend(&mut self, data: &[u8]) -> postcard::Result<()> {
        self.push_slice(data).map_err(|()| postcard::Error::SerializeBufferFull)
    }
}

impl<const L: usize> SerFlavor for &mut Fifo<u8, L> {
    type Output = Self;

    fn try_push(&mut self, data: u8) -> postcard::Result<()> {
        self.push(data).map_err(|()| postcard::Error::SerializeBufferFull)
    }

    fn finalize(self) -> postcard::Result<Self::Output> {
        Ok(self)
    }

    fn try_extend(&mut self, data: &[u8]) -> postcard::Result<()> {
        self.push_slice(data).map_err(|()| postcard::Error::SerializeBufferFull)
    }
}

#[derive(Debug)]
pub struct DynFifoBorrow<'f, const L: usize>(pub RefMut<'f, Fifo<u8, L>>);

impl<'f, const L: usize> Index<usize> for DynFifoBorrow<'f, L> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &(*self.0)[index]
    }
}

impl<'f, const L: usize> IndexMut<usize> for DynFifoBorrow<'f, L> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut (*self.0)[index]
    }
}

impl<const L: usize> SerFlavor for DynFifoBorrow<'_, L> {
    type Output = Self;

    fn try_push(&mut self, data: u8) -> postcard::Result<()> {
        (*self.0).try_push(data)
    }

    fn finalize(self) -> postcard::Result<Self::Output> {
        Ok(self)
    }
}

impl<'f, const L: usize> Extend<u8> for DynFifoBorrow<'f, L> {
    fn extend<It: IntoIterator<Item = u8>>(&mut self, iter: It) {
        (*self.0).extend(iter)
    }
}

pub use encode::*;
pub use decode::*;
