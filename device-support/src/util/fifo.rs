//! Stack allocated FIFO. (TODO)

use core::{
    convert::{AsMut, AsRef},
    fmt::{self, Debug},
    iter::{ExactSizeIterator, FusedIterator, Iterator},
    mem::{replace, size_of, transmute, transmute_copy, MaybeUninit},
    ops::{Index, IndexMut},
};

// Note: Capacity is a constant so that the transition to const generics (once
// that lands on stable) will be not terrible painful.

// TODO: const generics!

pub(super) mod fifo_config {
    use core::mem::size_of;

    pub const DEFAULT_CAPACITY: usize = 256;
    pub type Cur = u16;

    // If this doesn't hold, the as in the next check isn't guaranteed not to
    // lose bits.
    sa::const_assert!(size_of::<Cur>() <= size_of::<usize>());

    // `FifoConfig::DEFAULT_CAPACITY` ∈ [1, Cur::MAX]
    sa::const_assert!(DEFAULT_CAPACITY <= Cur::max_value() as usize);
    sa::const_assert!(DEFAULT_CAPACITY >= 1);
}

pub use fifo_config::{DEFAULT_CAPACITY, Cur};

pub struct Fifo<T, const LEN: usize = DEFAULT_CAPACITY> {
    data: [MaybeUninit<T>; LEN],
    length: usize,
    /// Points to the next slot that holds data.
    /// Valid when `length` > 0.
    starting: Cur,
    /// Points to the next empty slot.
    /// Valid when `length` < CAPACITY.
    ending: Cur,
}

impl<T: Debug, const L: usize> Debug for Fifo<T, L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // write!(f, "Fifo<{}> {{ ", core::any::type_name::<T>())?;
        write!(f, "{} {{ ", core::any::type_name::<Self>())?;

        let (a, b) = self.as_slice();
        let contents = a.iter().chain(b.iter());

        if self.length() >= 15 {
            for elem in contents.clone().take(7) {
                elem.fmt(f)?;
                write!(f, ", ")?;
            }

            write!(f, "...")?;

            for elem in contents.skip(self.length() - 7) {
                write!(f, ", ")?;
                elem.fmt(f)?;
            }
        } else {
            let mut iter = contents.take(self.length() - 1);
            for elem in &mut iter {
                elem.fmt(f)?;
                write!(f, ", ")?;
            }

            if let Some(last) = iter.next() {
                last.fmt(f)?;
            }
        }

        write!(f, " }}")
    }
}

impl<T, const L: usize> Default for Fifo<T, L> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const LEN: usize> Fifo<T, LEN> {
    /// Creates an empty `Fifo`.
    pub const fn new() -> Self {
        // If this doesn't hold, the as in the next check isn't guaranteed not to
        // lose bits.
        sa::const_assert!(size_of::<Cur>() <= size_of::<usize>());

        // `LEN` ∈ [1, Cur::MAX]
        assert!(LEN <= Cur::max_value() as usize);
        assert!(LEN >= 1);


        // This is really a job for `MaybeUninit::uninit_array` but alas, it is
        // not yet stable (it needs const generics).
        let data = MaybeUninit::<[MaybeUninit<T>; LEN]>::uninit();

        // This is safe because we can assume that _arrays_ have the same memory
        // representation as a literal composite of their elements (so an array
        // of MaybeUninits contains only the bits belonging to the MaybeUninit
        // elements: there aren't other bits we need to worry about) and because
        // we can then safely call `assume_init` on any `MaybeUninit<_>` type
        // since `MaybeUninit<_>` (which is aware that the type inside it may
        // not be valid for the bit representation it current has) is valid for
        // all bit representations.
        #[allow(unsafe_code)]
        let data = unsafe { data.assume_init() };

        Self {
            data,
            length: 0,
            starting: 0,
            ending: 0,
        }
    }

    /// The maximum number of elements the `Fifo` can hold.
    pub const fn capacity(&self) -> usize {
        LEN
    }

    /// Whether the `Fifo` is empty or not.
    pub const fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Whether the `Fifo` is full or not.
    pub const fn is_full(&self) -> bool {
        self.length == LEN
    }

    /// Number of elements currently in the `Fifo`.
    pub const fn length(&self) -> usize {
        self.length
    }

    /// Number of open slots the `Fifo` currently has.
    pub const fn remaining(&self) -> usize {
        LEN - self.length
    }

    // A wheel function.
    // Note: this is not overflow protected!
    // TODO: spin off the protected wheel into its own crate and use that
    // here!
    const fn add(pos: Cur, num: Cur) -> Cur {
        // Note: usize is guaranteed to be ≥ to Cur in size so the cast is
        // guaranteed not to lose bits.
        (((pos as usize) + (num as usize)) % LEN) as Cur
    }

    /// Adds a value to the `Fifo`, if possible.
    ///
    /// Returns `Err(())` if the `Fifo` is currently full.
    #[inline]
    pub fn push(&mut self, datum: T) -> Result<(), ()> {
        if self.is_full() {
            Err(())
        } else {
            self.length += 1;
            self.data[self.ending as usize] = MaybeUninit::new(datum);
            self.ending = Self::add(self.ending, 1);

            Ok(())
        }
    }

    /// Gives a reference to the next value in the `Fifo`, if available.
    ///
    /// This function doesn't remove the value from the `Fifo`; use `pop` to do
    /// that.
    ///
    /// [`pop`]: Fifo::pop
    #[inline]
    pub fn peek(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            let datum: *const T = self.data[self.starting as usize].as_ptr();

            // Leaning on our invariants here; if we haven't just returned this
            // specific value was inserted (in a valid state) so we can safely
            // assume that this value is initialized.
            #[allow(unsafe_code)]
            Some(unsafe { &*datum })
        }
    }

    // Updates the starting and length count to 'consume' some number of
    // elements.
    fn advance(&mut self, num: Cur) {
        debug_assert!((num as usize) <= self.length);

        self.length -= num as usize;
        self.starting = Self::add(self.starting, num);
    }

    /// Pops a value from the Fifo, if available.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if Self::is_empty(self) {
            None
        } else {
            let datum = replace(
                &mut self.data[self.starting as usize],
                MaybeUninit::uninit(),
            );

            self.advance(1);

            // As with peek, we trust our invariants to keep us safe here.
            // Because this value must have been initialized for us to get here,
            // we know this is safe.
            #[allow(unsafe_code)]
            Some(unsafe { datum.assume_init() })
        }
    }

    /// Returns a mutable slice consisting of the data currently in the `Fifo`
    /// without removing it.
    #[inline]
    pub fn as_mut_slice(&mut self) -> (&mut [T], &mut [T]) {
        // starting == ending can either mean a full fifo or an empty one so
        // we use our length field to handle this case separately
        if Fifo::is_empty(self) {
            (&mut [], &mut [])
        } else {
            if self.ending > self.starting {
                let s = &mut self.data
                    [(self.starting as usize)..(self.ending as usize)];

                // Again, leaning on our invariants and assuming this is all
                // init-ed data.
                // TODO: not confident the transmute is actually safe here.
                //
                // [MaybeUninit<T>] and [T]
                // and
                // &[MaybeUninit<T>] and &[T]
                // have the same representations right?
                //
                // This is probably safe since `MaybeUninit<T>` and `T` are
                // guaranteed to have the same representation (size, alignment,
                // and ABI).
                // Probably because as per the `MaybeUninit` union docs, types
                // that contain a MaybeUninit don't necessarily have to have the
                // same representation as types that just contain `T`. There's
                // an assert for this at the bottom of this file.
                #[allow(unsafe_code)]
                let s = unsafe {
                    transmute(s)
                };

                (s, &mut [])
            } else if self.ending <= self.starting {
                // Gotta do it in two parts then.
                let s = &mut self.data[(self.starting as usize)..];

                // Same as above.
                #[allow(unsafe_code)]
                let s = unsafe {
                    transmute(s)
                };

                let e = &mut self.data[..(self.ending as usize)];
                #[allow(unsafe_code)]
                let e = unsafe {
                    transmute(e)
                };

                (s, e)
            } else {
                unreachable!()
            }
        }
    }

    /// Returns a slice consisting of the data currently in the `Fifo` without
    /// removing it.
    #[inline]
    pub fn as_slice(&self) -> (&[T], &[T]) {
        // This contains the exact logic from as_mut_slice above.
        // TODO: is there a way to avoid duplicating this?

        if self.is_empty() {
            (&[], &[])
        } else {
            if self.ending > self.starting {
                let f = &self.data
                    [(self.starting as usize)..(self.ending as usize)];

                #[allow(unsafe_code)]
                let f = unsafe {
                    transmute(f)
                };

                (f, &[])
            } else if self.ending <= self.starting {
                // Gotta do it in two parts then.
                let s = &self.data[(self.starting as usize)..];

                #[allow(unsafe_code)]
                let s = unsafe {
                    transmute(s)
                };

                let e = &self.data[..(self.ending as usize)];
                #[allow(unsafe_code)]
                let e = unsafe {
                    transmute(e)
                };

                (s, e)
            } else {
                unreachable!()
            }
        }
    }
}

impl<T: Clone, const L: usize> Fifo<T, L> {
    /// Because we cannot take ownership of the slice, this is only available
    /// for `Clone` (and, thus, `Copy`) types.
    ///
    /// This operation is 'atomic': either all of the slice gets pushed (if
    /// there is space for it) or none of it does.
    ///
    /// If the slice cannot be pushed in its entirety, this function returns
    /// `Err(())`.
    pub fn push_slice(&mut self, slice: &[T]) -> Result<(), ()> {
        if self.remaining() < slice.len() {
            Err(())
        } else {
            for v in slice.iter().cloned() {
                self.push(v).expect("fifo: internal error")
            }

            Ok(())
        }
    }
}

impl<T, const L: usize> Fifo<T, L> {
    /// Like [`push_slice`] this function is 'atomic': it will either succeed
    /// (and in this case push the iterator in its entirety) or it will leave
    /// the `Fifo` unmodified.
    ///
    /// Because we want this property we need to know the length of the iterator
    /// beforehand and that's where [`ExactSizeIterator`] comes in. With a
    /// normal [`Iterator`] we can't know the length of the iterator until we've
    /// consumed it, but `ExactSizeIterator`s just tells us.
    ///
    /// This particular function will require an iterator that transfers
    /// ownership of the values (i.e the kind you get when you call [`drain`] on
    /// a [`Vec`]). If this is not what you want (and if your type is
    /// [`Clone`]able), try [`push_iter_ref`].
    ///
    /// Like [`push_slice`], this will return `Err(())` if it is unable to push
    /// the entire iterator. Note that we take a mutable reference to your
    /// iterator, so in the event that we are not able to push your values, they
    /// are not just dropped (you can try again or do something else with your
    /// values).
    ///
    /// [`push_slice`]: Fifo::push_slice
    /// [`push_iter_ref`]: Fifo::push_iter_ref
    /// [`ExactSizeIterator`]: core::iter::ExactSizeIterator
    /// [`Iterator`]: core::iter::Iterator
    /// [`Clone`]: core::clone::Clone
    /// [`Vec`]: alloc::vec::Vec
    /// [`drain`]: alloc::vec::Vec::drain
    pub fn push_iter<I: ExactSizeIterator<Item = T>>(
        &mut self,
        iter: &mut I,
    ) -> Result<(), ()> {
        let len = iter.len();

        if self.remaining() < len {
            Err(())
        } else {
            for _ in 0..len {
                self.push(
                    iter.next().expect("ExactSizeIterator length was wrong!"),
                )
                .expect("fifo: internal error")
            }

            Ok(())
        }
    }
}

impl<'a, T: Clone + 'a, const L: usize> Fifo<T, L> {
    /// The version of [`push_iter`] that doesn't need ownership of the `T`
    /// values your iterator is yielding.
    ///
    /// This works like [`push_slice`] does and thus this also only works for
    /// types that implement [`Clone`].
    ///
    /// Returns `Err(())` if unable to push the entire iterator.
    ///
    /// [`push_iter`]: Fifo::push_iter
    /// [`push_slice`]: Fifo::push_slice
    /// [`Clone`]: core::clone::Clone
    pub fn push_iter_ref<'i: 'a, I: ExactSizeIterator<Item = &'a T>>(
        &mut self,
        iter: &'i mut I,
    ) -> Result<(), ()> {
        self.push_iter(&mut iter.cloned())
    }
}

impl<T: Clone, const L: usize> Fifo<T, L> {
    /// Useful for generating arrays out of a `Clone`able (but not `Copy`able)
    /// value to pass into `Fifo::push_slice`.
    pub fn array_init_using_clone(val: T) -> [T; L] {
        // MaybeUninit is always properly initialized.
        // Note: this is _the_ use case for `MaybeUninit::uninit_array` which is
        // not yet stable (blocked on const-generics like all the shiny things).
        #[allow(unsafe_code)]
        let mut inner: [MaybeUninit<T>; L] =
            unsafe { MaybeUninit::uninit().assume_init() };

        for elem in &mut inner[..] {
            *elem = MaybeUninit::new(val.clone());
        }

        debug_assert_eq!(
            size_of::<[MaybeUninit<T>; L]>(),
            size_of::<[T; L]>()
        );

        // Because we've initialized every element manually, this is safe.
        // Additionally, the assert above (which will always be true in our
        // case) is a way for us to be extremely certain that `transmute_copy`'s
        // invariant is upheld.
        #[allow(unsafe_code)]
        unsafe {
            transmute_copy(&inner)
        }
    }
}

impl<T> Index<usize> for Fifo<T> {
    type Output = T;

    #[inline]
    fn index(&self, idx: usize) -> &T {
        &self.as_slice()[idx]
    }
}

impl<T> IndexMut<usize> for Fifo<T> {
    #[inline]
    fn index_mut(&mut self, idx: usize) -> &mut T {
        &mut self.as_mut_slice()[idx]
    }
}

impl<T> AsRef<[T]> for Fifo<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, const L: usize> AsMut<[T]> for Fifo<T, L> {
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

// Use `Iterator::by_ref` to retain ownership of the iterator
impl<T, const L: usize> Iterator for Fifo<T, L> { // /*&mut */
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        self.pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.length(), Some(self.length()))
    }
}

impl<T, const L: usize> FusedIterator for Fifo<T, L> {}

impl<T, const L: usize> ExactSizeIterator for Fifo<T, L> {}

using_alloc! {
    use core::convert::TryInto;

    use bytes::{{Buf, BufMut}, buf::UninitSlice};

    impl Buf for Fifo<u8> {
        fn remaining(&self) -> usize {
            self.length()
        }

        fn chunk(&self) -> &[u8] {
            self.as_slice()
        }

        fn advance(&mut self, count: usize) {
            self.advance(count.try_into().unwrap());
        }
    }

    unsafe impl BufMut for Fifo<u8> {
        fn remaining_mut(&self) -> usize {
            self.remaining()
        }

        #[allow(unsafe_code)] // Nothing _we_ do here is unsafe..
        unsafe fn advance_mut(&mut self, cnt: usize) {
            if cnt > self.remaining_mut() {
                panic!("Attempted to write more than the buffer can accommodate.");
            }

            // If cnt is less than the number of slots we've got and the number of
            // slots we've got is representable by the cursor size, this should be
            // fine.
            let cnt_cur: Cur = cnt.try_into().unwrap();

            // Should also be fine (for overflow) if the check above doesn't panic.
            // We also won't exceed the capacity of the fifo if we're not writing
            // more than number of slots that are remaining (the above check).
            self.length += cnt;
            self.ending = Self::add(self.ending, cnt_cur);
        }

        fn chunk_mut(&mut self) -> &mut UninitSlice {
            fn into_uninit_slice(m: &mut [MaybeUninit<u8>]) -> &mut UninitSlice {
                let ptr = m as *mut _ as *mut u8;
                let len = m.len();

                unsafe { UninitSlice::from_raw_parts_mut(ptr, len) }
            }

            if Self::is_empty(self) {
                into_uninit_slice(&mut self.data)
            } else {
                if self.ending <= self.starting {
                    into_uninit_slice(&mut self.data[(self.ending as usize)..(self.starting as usize)])
                } else if self.ending > self.starting {
                    // Gotta do it in two parts then.
                    into_uninit_slice(&mut self.data[(self.ending as usize)..])
                } else { unreachable!() }
            }
        }
    }
}

// Note: if we switch to const generics for `CAPACITY`, move this to the
// constructor.
sa::assert_eq_size!(&mut [MaybeUninit<u8>], &mut [u8]);
sa::assert_eq_size!([MaybeUninit<u8>; DEFAULT_CAPACITY], [u8; DEFAULT_CAPACITY]);
sa::assert_eq_align!([MaybeUninit<u8>; DEFAULT_CAPACITY], [u8; DEFAULT_CAPACITY]);

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const FIFO: Fifo<usize> = Fifo::new();

    // A type that implements Clone (but not Copy!).
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Cloneable {
        a: usize,
        b: usize,
        c: usize,
    }

    impl Cloneable {
        const fn new(n: usize) -> Self {
            Self { a: n, b: n, c: n }
        }
    }

    // A type this does *not* implement Clone.
    #[derive(Debug, PartialEq, Eq)]
    struct Uncloneable {
        inner: Cloneable,
    }

    impl Uncloneable {
        const fn new(n: usize) -> Self {
            Self {
                inner: Cloneable::new(n),
            }
        }
    }

    // Also tests push_slice (requires Clone)
    #[test]
    fn new_with_values() {
        let c = Cloneable::new(78);
        let arr = <Fifo<_>>::array_init_using_clone(c.clone());
        let mut fifo = <Fifo<_>>::new();

        assert_eq!(Ok(()), fifo.push_slice(&arr));

        assert_eq!(fifo.length(), fifo.capacity());

        let mut count = 0;
        for i in fifo.by_ref() {
            assert_eq!(c, i);
            count += 1;
        }

        assert_eq!(count, fifo.capacity());
    }

    const BIG_SLICE: [u8; DEFAULT_CAPACITY + 2] = [0; DEFAULT_CAPACITY + 2];

    #[test]
    fn push_slice_too_big() {
        let mut fifo = <Fifo<_>>::new();

        assert_eq!(0, fifo.length());
        assert_eq!(Err(()), fifo.push_slice(&BIG_SLICE));
        assert_eq!(0, fifo.length());
    }

    // Tests pushing cloneable values in an iterator
    #[test]
    fn push_iter_ref() {
        let mut fifo = <Fifo<_>>::new();

        macro_rules! ascii_string {
            ($($c:literal)*) => {
                [$(
                    $c as u8
                ),*]
            };
        }

        let string = ascii_string!['H''e''l''l''o'' ''W''o''r''l''d''!'];

        assert_eq!(Ok(()), fifo.push_iter_ref(&mut string.iter()));

        for (idx, c) in fifo.enumerate() {
            assert_eq!(string[idx], c);
        }
    }

    // Tests pushing an owned iterator
    #[test]
    fn push_uncloneable() {
        let mut arr = [0; DEFAULT_CAPACITY];
        for i in 0..DEFAULT_CAPACITY {
            arr[i] = i;
        }

        let mut iter = arr.iter().map(|i| Uncloneable::new(*i));
        let mut fifo = <Fifo<_>>::new();

        assert_eq!(Ok(()), fifo.push_iter(&mut iter));

        for (idx, uc) in fifo.enumerate() {
            assert_eq!(Uncloneable::new(idx), uc);
        }
    }

    #[test]
    fn overpush() {
        let mut fifo = FIFO;

        for i in 0..fifo.capacity() {
            assert_eq!(Ok(()), fifo.push(i));
        }

        assert_eq!(Err(()), fifo.push(123));
        assert_eq!(Err(()), fifo.push(567));
    }

    #[test]
    fn overpop() {
        let mut fifo = FIFO;

        assert_eq!(None, fifo.pop());
        assert_eq!(None, fifo.pop());

        for i in 0..fifo.capacity() {
            assert_eq!(Ok(()), fifo.push(i));
        }

        // Also tests ordering!
        for i in 0..fifo.capacity() {
            assert_eq!(Some(i), fifo.pop());
        }

        assert_eq!(None, fifo.pop());
        assert_eq!(None, fifo.pop());
    }

    #[test]
    fn peek() {
        let mut fifo = <Fifo<_>>::new();

        assert_eq!(Ok(()), fifo.push(278));
        assert_eq!(Ok(()), fifo.push(513));
        assert_eq!(2, fifo.length());

        assert_eq!(Some(&278), fifo.peek());
        assert_eq!(2, fifo.length());

        assert_eq!(Some(278), fifo.pop());
        assert_eq!(1, fifo.length());

        assert_eq!(Some(&513), fifo.peek());
        assert_eq!(1, fifo.length());

        assert_eq!(Some(513), fifo.pop());
        assert_eq!(0, fifo.length());

        assert_eq!(None, fifo.pop());
    }

    #[test]
    fn by_ref() {
        let mut f = Fifo::<u8>::new();

        assert_eq!(Ok(()), f.push_iter(&mut (0..10)));
        assert_eq!(10, f.length());

        // If we remove by_ref here this will fail to compile because `f` will
        // be _consumed_.
        for (idx, i) in f.by_ref().enumerate() {
            assert_eq!(idx as u8, i);
        }

        assert_eq!(0, f.length());
    }
}
