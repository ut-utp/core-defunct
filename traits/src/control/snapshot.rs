//! The [`Snapshot` trait](crate::control::Snapshot).
//!
//! Allows for a type's state to be recorded and for a recorded state to
//! be restored.
//!
//! TODO!

use core::clone::Clone;
use core::convert::Infallible;
use core::fmt::{Debug, Display};

// TODO
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SnapshotError {
    UnrecordableState,
    UninterruptableState,
    Other(&'static str), // TODO: this should perhaps be a &dyn Debug or something
}

impl From<Infallible> for SnapshotError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl Display for SnapshotError {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // TODO
        unimplemented!()
    }
}

using_std! { impl std::error::Error for SnapshotError { } }

#[ambassador::delegatable_trait]
pub trait Snapshot {
    type Snap;
    type Err: Debug + Into<SnapshotError>;

    // Is fallible; can fail if the simulator is in a state that can't be
    // snapshotted.
    fn record(&self) -> Result<Self::Snap, Self::Err>;

    // This is also fallible. This can fail if the simulator is not in a state
    // where the current state can be abandoned for an old state.
    fn restore(&mut self, snap: Self::Snap) -> Result<(), Self::Err>;
}

// We'd like to offer this but this blanket impl makes things annoying for
// wrapper types that would like to conditionally implement `Clone` (but not
// *require* that their contents implement `Snapshot` by way of implementing
// `Clone`).
//
// I'm also not sure that it's always correct/desirable to use `Clone` as your
// `Snapshot` impl; we don't want to have users need to "choose" between
// implementing `Snapshot` and `Clone.
/* impl<T: Clone> Snapshot for T {
    type Snap = Self;
    type Err = Infallible;

    fn record(&self) -> Result<Self::Snap, Self::Err> {
        Ok(self.clone())
    }

    fn restore(&mut self, snap: Self) -> Result<(), Self::Err> {
        *self = snap;

        Ok(())
    }
}
*/

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct SnapshotUsingClone<T: Clone>(pub T);
impl<T: Clone> Snapshot for SnapshotUsingClone<T> {
    type Snap = T;
    type Err = Infallible;

    fn record(&self) -> Result<Self::Snap, Self::Err>  {
        Ok(self.0.clone())
    }

    fn restore(&mut self,snap:Self::Snap) -> Result<(), Self::Err>  {
        self.0 = snap;

        Ok(())
    }
}

trait SnapshotExt {
    fn snapshot_using_clone(self) -> SnapshotUsingClone<Self> where Self: Clone {
        SnapshotUsingClone(self)
    }
}

impl<T> SnapshotExt for T { }

// todo: delegate all traits to `SnapshotUsingClone`! (Memory included)
