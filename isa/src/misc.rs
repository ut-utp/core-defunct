//! Miscellaneous odds and ends that are loosely related to the LC-3 ISA.
//!
//! (TODO!)


pub mod util {
    /// Associated types and other weird bits for the LC-3 ISA.
    use crate::{Addr, Word, ADDR_SPACE_SIZE_IN_WORDS};

    use core::ops::{Deref, DerefMut};

    // TODO: on `std` impl `MemoryDump` from `io::Read`?

    // Newtype
    #[derive(Clone)] // TODO: impl Debug + PartialEq/Eq + Ser/De + Hash
    pub struct MemoryDump(pub [Word; ADDR_SPACE_SIZE_IN_WORDS]);
    impl Deref for MemoryDump {
        type Target = [Word; ADDR_SPACE_SIZE_IN_WORDS];

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for MemoryDump {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl From<[Word; ADDR_SPACE_SIZE_IN_WORDS]> for MemoryDump {
        fn from(memory: [Word; ADDR_SPACE_SIZE_IN_WORDS]) -> Self {
            Self(memory)
        }
    }

    impl MemoryDump {
        pub fn blank() -> Self {
            [0; ADDR_SPACE_SIZE_IN_WORDS].into()
        }

        pub fn layer_loadable<L: LoadableIterator>(&mut self, loadable: L) -> &mut Self {
            for (addr, word) in loadable {
                self[addr as usize] = word;
            }

            self
        }

        // TODO: provide a trait for this too
        // TODO: does it make sense to impl FromIterator for any of these types?
        pub fn layer_iterator<I: Iterator<Item = (Addr, Word)>>(&mut self, iter: I) -> &mut Self {
            for (addr, word) in iter {
                self[addr as usize] = word;
            }

            self
        }
    }

    type AssembledProgramInner = [(Word, bool); ADDR_SPACE_SIZE_IN_WORDS];

    impl From<AssembledProgram> for MemoryDump {
        fn from(memory: AssembledProgram) -> Self {
            let mut mem: [Word; ADDR_SPACE_SIZE_IN_WORDS] = [0; ADDR_SPACE_SIZE_IN_WORDS];

            memory
                .iter()
                .enumerate()
                .for_each(|(idx, (w, _))| mem[idx] = *w);

            Self(mem)
        }
    }

    impl From<AssembledProgramInner> for MemoryDump {
        fn from(memory: AssembledProgramInner) -> Self {
            Into::<AssembledProgram>::into(memory).into()
        }
    }

    // Newtype
    #[derive(Clone)] // TODO: impl Debug + PartialEq/Eq + Ser/De + Hash
    pub struct AssembledProgram(pub [(Word, bool); ADDR_SPACE_SIZE_IN_WORDS]);
    impl AssembledProgram {
        pub const fn new(mem: [(Word, bool); ADDR_SPACE_SIZE_IN_WORDS]) -> Self {
            Self(mem)
        }
    }

    impl Deref for AssembledProgram {
        type Target = AssembledProgramInner;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for AssembledProgram {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl From<AssembledProgramInner> for AssembledProgram {
        fn from(prog: AssembledProgramInner) -> Self {
            Self(prog)
        }
    }

    // pub trait LoadableIterator<'a>: IntoIterator<Item = &'a (Addr, Word)> + Sized {
    pub trait LoadableIterator: IntoIterator<Item = (Addr, Word)> + Sized {
        fn to_memory_dump(self) -> MemoryDump {
            let mut mem: [Word; ADDR_SPACE_SIZE_IN_WORDS] = [0; ADDR_SPACE_SIZE_IN_WORDS];

            self.into_iter()
                .for_each(|(addr, word)| mem[addr as usize] = word);

            mem.into()
        }
    }

    impl<I: IntoIterator<Item = (Addr, Word)>> LoadableIterator for I {}

    use core::{
        iter::{Enumerate, Filter, Map},
        slice::Iter,
    };

    impl<'a> IntoIterator for &'a MemoryDump {
        type Item = (Addr, Word);
        // type IntoIter = MemoryDumpLoadableIterator<'a>;
        type IntoIter = Map<Enumerate<Iter<'a, Word>>, &'a dyn Fn((usize, &Word)) -> (Addr, Word)>;

        fn into_iter(self) -> Self::IntoIter {
            self.iter()
                .enumerate()
                .map(&|(idx, word)| (idx as Addr, *word))
        }
    }

    impl<'a> IntoIterator for &'a AssembledProgram {
        type Item = (Addr, Word);
        type IntoIter = Map<
            Filter<Enumerate<Iter<'a, (Word, bool)>>, &'a dyn Fn(&(usize, &(u16, bool))) -> bool>,
            &'a dyn Fn((usize, &(Word, bool))) -> (Addr, Word),
        >;

        #[allow(trivial_casts)]
        fn into_iter(self) -> Self::IntoIter {
            self.iter()
                .enumerate()
                .filter(
                    (&|(_, (_, set)): &(usize, &(Word, bool))| *set)
                        as &(dyn Fn(&(usize, &(u16, bool))) -> bool),
                ) // This cast is marked as trivial but it's not, apparently
                .map(&|(idx, (word, _)): (usize, &(Word, bool))| (idx as Addr, *word))
        }
    }
}
