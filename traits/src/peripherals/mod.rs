//! Peripherals! The [`Peripherals` supertrait](peripherals::Peripherals) and the rest of the
//! peripheral and device traits.

pub mod adc;
pub mod clock;
pub mod gpio;
pub mod pwm;
pub mod timers;
pub mod input;
pub mod output;

pub use gpio::Gpio;
pub use adc::Adc;
pub use pwm::Pwm;
pub use timers::Timers;
pub use clock::Clock;
pub use input::Input;
pub use output::Output;

pub mod stubs;

////////////////////////////////////////////////////////////////////////////////

#[macro_use] mod support;

use core::{
    cell::RefCell,
    convert::Infallible,
};

use super::control::snapshot::{SnapshotUsingClone, Snapshot, SnapshotError};
use crate::*;
use self::{gpio::*, adc::*, pwm::*, timers::*, input::*, output::*};
use lc3_isa::Word;
use support::{
    delegated_peripheral_impl_support::{self, *},
    optional_peripheral_support::{Bool, OptTy, OptPresent, OptionalPeripheral, Present, NotPresent},
};


// the reason this trait exists is to accommodate implementations that wish to
// share state between peripherals
//
// for all but the most extreme use cases you should not have to implement this trait manually; using
// `PeripheralSet` is what you want
//
// but for use cases where you want to share state between peripherals, `PeripheralSet`,
// which requires separate instances for each peripheral, makes this difficult to do without
// resorting to using types with shared interior mutability (i.e. `RefCell`, `Arc<Mutex<_>>`,
// etc.)
//
// consider the following example:
// We want to offer `Input` and `Output` peripheral implementations that are linked (loopback
// mode); i.e. anything that's sent to the Output peripheral shows up as an Input.
//
// ```
// ```
//
// In this specific case, because the data being "shared" is `Copy`, we can use `Cell` instead
// of `RefCell`, reducing the runtime cost of sharing data between the `LoopbackInput` and `LoopbackOutput`
// instances.
//
// However in cases where the shared data is not `Copy`, you'll need to resort to `RefCell` or `Arc<Mutex<_>>`
// or another heavier wrapper. This is unfortunate because the `Input` and `Output` peripherals are ultimately never used
// concurrently; we do not actually _need_ to synchronize access to data they share.
//
// More importantly there's an ergonomic cost to being forced to fit the "one instance per peripheral" paradigm: `LoopbackInput`
// and `LoopbackOutput` need to grow a lifetime parameter and now cannot have implementations of `Default`.
//
// Consider this alternative implementation:
// ```
// struct LoopbackIo
// ```
//
// This implementation is more natural to write, has less overhead, and is easier to use (no lifetime parameter, it can implement Default, implements `Send` and `Sync`).
//
// Astute readers will notice that it is still _possible_ to use `LoopbackIo` with `PeripheralSet` – just with a little extra runtime cost!
// We can make use of the `Arc<RwLock<_>>` blanket impls for `Input` and `Output` to provide `PeripheralSet` with `Input` and `Output` instances
// that actually both point to the same underlying `LoopbackIo` instance.
// ```
//
// ```
//
// The downside, of course, is runtime cost.
//
// This nicely demonstrates the initial claim we made: for all but the _most extreme_ use cases, you're likely best served by `PeripheralSet`.
// In the rare case you want to share state between peripherals **and** cannot accept extra runtime cost (i.e. because you're on embedded or because you
// don't have an allocator) or worse ergonomics, the `Peripherals` trait is the escape hatch you're looking for.

peripherals! {
/// TODO!
pub trait Peripherals = {
    required: {
        gpio:   Gpio   (G),
        adc:    Adc    (A),
        pwm:    Pwm    (P),
        timers: Timers (T),
        clock:  Clock  (C),
        input:  Input  (I),
        output: Output (O),
    },

    optional: {
        gpio_bank_b: Gpio (GB) as GpioB = NoGpio
            where (stubs::GpioStub) is MissingGpio wrapped as NoGpio,
        gpio_bank_c: Gpio (GC) as GpioC = NoGpio,
    },
} with {
    /// TODO!
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[derive(serde::Serialize, serde::Deserialize)]
    set = struct PeripheralSet<...>;

    /// TODO!
    // TODO: docs explaining that the utility here is getting a type that
    // has delegated impls of all the peripheral traits + `Peripherals` implemented!
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[derive(serde::Serialize, serde::Deserialize)]
    wrapper = struct PeripheralsWrapper<_>;
}}

pub trait PeripheralsExt: Peripherals {
    /// Gets you a wrapper type that impls all the traits.
    fn get_peripherals_wrapper(&self) -> &PeripheralsWrapper<Self> {
        // SAFETY: `PeripheralsWrapper` is `repr(transparent)` and we're
        // constructing a shared reference out of a shared reference of the same
        // lifetime.
        unsafe { core::mem::transmute(self) }
    }
    fn get_peripherals_wrapper_mut(&mut self) -> &mut PeripheralsWrapper<Self> {
        // SAFETY: `PeripheralsWrapper` is `repr(transparent)`, we're using a
        // mutable reference to construct a mutable reference with the same
        // lifetime.
        unsafe { core::mem::transmute(self) }
    }
}

impl<P: Peripherals> PeripheralsExt for P { }

// TODO: hide the ambassador items in the docs!

/* struct LoopbackInput<'a> {
    char: &'a Cell<Option<u8>>,
    interrupts: bool,
}

impl Input for LoopbackInput {
    fn read_data(&self) -> Result<u8, input::InputError> {
        self.char.take().ok_or(input::InputError::NoDataAvailable)
    }

    fn current_data_unread(&self) -> bool {
        self.char.get().is_some()
    }

    fn register_interrupt_flag(&mut self,flag: & 'a core::sync::atomic::AtomicBool) {
        todo!()
    }

    fn interrupt_occurred(&self) -> bool {
        self.current_data_unread()
    }

    fn reset_interrupt_flag(&mut self,) {
        todo!()
    }

    fn set_interrupt_enable_bit(&mut self,bit:bool) {
        self.interrupts = bit;
    }

    fn interrupts_enabled(&self) -> bool {
        self.interrupts
    }
}

struct LoopbackOutput<'a> {
    char: &'a Cell<Option<u8>>,
    interrupts: bool,
} */
