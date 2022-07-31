//! Peripherals! The [`Peripherals` supertrait](peripherals::Peripherals) and the rest of the
//! peripheral and device traits.

pub mod adc;
pub mod clock;
pub mod gpio;
pub mod pwm;
pub mod timers;
pub mod input;
pub mod output;

use core::{marker::PhantomData, convert::Infallible};

pub use gpio::Gpio;
pub use adc::Adc;
pub use pwm::Pwm;
pub use timers::Timers;
pub use clock::Clock;
pub use input::Input;
pub use output::Output;

pub mod stubs;


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
// Astute readers will notice that it is still _possible_ to use `LoopbackIo` with `PeripheralsSet` – just with a little extra runtime cost!
// We can make use of the `Arc<RwLock<_>>` blanket impls for `Input` and `Output` to provide `PeripheralsSet` with `Input` and `Output` instances
// that actually both point to the same underlying `LoopbackIo` instance.
// ```
//
// ```
//
// The downside, of course, is runtime cost.
//
// This nicely demonstrates the initial claim we made: for all but the _most extreme_ use cases, you're likely best served by `PeripheralsSet`.
// In the rare case you want to share state between peripherals **and** cannot accept extra runtime cost (i.e. because you're on embedded or because you
// don't have an allocator) or worse ergonomics, the `Peripherals` trait is the escape hatch you're looking for.
#[ambassador::delegatable_trait]
pub trait Peripherals {
    type Gpio: Gpio;
    type Adc: Adc;
    type Pwm: Pwm;
    type Timers: Timers;
    type Clock: Clock;
    type Input: Input;
    type Output: Output;

    type GpioB: Gpio/*  = gpio::MissingGpio */;
    type GpioC: Gpio/*  = gpio::MissingGpio */;

    fn get_gpio(&self) -> &Self::Gpio;
    fn get_gpio_mut(&mut self) -> &mut Self::Gpio;

    fn get_adc(&self) -> &Self::Adc;
    fn get_adc_mut(&mut self) -> &mut Self::Adc;

    fn get_pwm(&self) -> &Self::Pwm;
    fn get_pwm_mut(&mut self) -> &mut Self::Pwm;

    fn get_timers(&self) -> &Self::Timers;
    fn get_timers_mut(&mut self) -> &mut Self::Timers;

    fn get_clock(&self) -> &Self::Clock;
    fn get_clock_mut(&mut self) -> &mut Self::Clock;

    fn get_input(&self) -> &Self::Input;
    fn get_input_mut(&mut self) -> &mut Self::Input;

    fn get_output(&self) -> &Self::Output;
    fn get_output_mut(&mut self) -> &mut Self::Output;

    // optional peripherals:
    fn get_gpio_bank_b(&self) -> Option<&Self::GpioB> { None }
    fn get_gpio_bank_b_mut(&mut self) -> Option<&mut Self::GpioB> { None }

    fn get_gpio_bank_c(&self) -> Option<&Self::GpioC> { None }
    fn get_gpio_bank_c_mut(&mut self) -> Option<&mut Self::GpioC> { None }
}

pub trait PeripheralsExt: Peripherals {
    /// Gets you a wrapper type that impls all the traits.
    fn get_peripherals_wrapper(&mut self) -> PeripheralsWrapper<'_, Self> { PeripheralsWrapper(self) }
}

impl<P: Peripherals> PeripheralsExt for P { }

#[derive(Debug)]

pub struct PeripheralsWrapper<'a, P: ?Sized + Peripherals + 'a>(&'a mut P);

// #[ambassador::delegate_to_methods]
// #[delegate(Peripherals, target_ref = "get", target_mut = "get_mut")]
// impl<'a, P: ?Sized + Peripherals> PeripheralsWrapper<'a, P> {
//     #[inline(always)]
//     fn get_mut(&mut self) -> &mut P { &mut self.0 }
//     #[inline(always)]
//     fn get(&self) -> &P { &self.0 }
// }

// TODO: fix in upstream
use crate::*;
use self::{gpio::*, adc::*, pwm::*, timers::*, input::*, output::*};
use lc3_isa::Word;

// TODO: doc spotlight the peripheral traits
#[ambassador::delegate_to_methods]
#[delegate(Gpio, target_ref = "get_gpio", target_mut = "get_gpio_mut")]
#[delegate(Adc, target_ref = "get_adc", target_mut = "get_adc_mut")]
#[delegate(Pwm, target_ref = "get_pwm", target_mut = "get_pwm_mut")]
#[delegate(Timers, target_ref = "get_timers", target_mut = "get_timers_mut")]
#[delegate(Clock, target_ref = "get_clock", target_mut = "get_clock_mut")]
#[delegate(Input, target_ref = "get_input", target_mut = "get_input_mut")]
#[delegate(Output, target_ref = "get_output", target_mut = "get_output_mut")]
impl<P: Peripherals> PeripheralsWrapper<'_, P> {
    fn get_gpio(&self) -> &P::Gpio { self.0.get_gpio() }
    fn get_gpio_mut(&mut self) -> &mut P::Gpio { self.0.get_gpio_mut() }
    fn get_adc(&self) -> &P::Adc { self.0.get_adc() }
    fn get_adc_mut(&mut self) -> &mut P::Adc { self.0.get_adc_mut() }
    fn get_pwm(&self) -> &P::Pwm { self.0.get_pwm() }
    fn get_pwm_mut(&mut self) -> &mut P::Pwm { self.0.get_pwm_mut() }
    fn get_timers(&self) -> &P::Timers { self.0.get_timers() }
    fn get_timers_mut(&mut self) -> &mut P::Timers { self.0.get_timers_mut() }
    fn get_clock(&self) -> &P::Clock { self.0.get_clock() }
    fn get_clock_mut(&mut self) -> &mut P::Clock { self.0.get_clock_mut() }
    fn get_input(&self) -> &P::Input { self.0.get_input() }
    fn get_input_mut(&mut self) -> &mut P::Input { self.0.get_input_mut() }
    fn get_output(&self) -> &P::Output { self.0.get_output() }
    fn get_output_mut(&mut self) -> &mut P::Output { self.0.get_output_mut() }
}

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

/*
pub struct PeripheralSet<G, A, P, T, C, I, O>
where
    G: Gpio,
    A: Adc,
    P: Pwm,
    T: Timers,
    C: Clock,
    I: Input,
    O: Output,
{
    gpio: G,
    adc: A,
    pwm: P,
    timers: T,
    clock: C,
    input: I,
    output: O,
}

impl<G, A, P, T, C, I, O> Default for PeripheralSet<G, A, P, T, C, I, O>
where
    G: Default + Gpio,
    A: Default + Adc,
    P: Default + Pwm,
    T: Default + Timers,
    C: Default + Clock,
    I: Default + Input,
    O: Default + Output,
{
    fn default() -> Self {
        Self {
            gpio: G::default(),
            adc: A::default(),
            pwm: P::default(),
            timers: T::default(),
            clock: C::default(),
            input: I::default(),
            output: O::default(),
        }
    }
}

impl<G, A, P, T, C, I, O> PeripheralSet<G, A, P, T, C, I, O>
where
    G: Gpio,
    A: Adc,
    P: Pwm,
    T: Timers,
    C: Clock,
    I: Input,
    O: Output,
{
    pub fn new(gpio: G, adc: A, pwm: P, timers: T, clock: C, input: I, output: O) -> Self {
        Self {
            gpio,
            adc,
            pwm,
            timers,
            clock,
            input,
            output,
        }
    }
}
*/

// TODO: doc spotlight the peripheral traits
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, ambassador::Delegate)]
#[delegate(Gpio, target = "gpio")]
#[delegate(Adc, target = "adc")]
#[delegate(Pwm, target = "pwm")]
#[delegate(Timers, target = "timers")]
#[delegate(Clock, target = "clock")]
#[delegate(Input, target = "input")]
#[delegate(Output, target = "output")]
pub struct PeripheralSet<G, A, P, T, C, I, O, GB = NoGpio, GC = NoGpio>
where
    G: Gpio,
    A: Adc,
    P: Pwm,
    T: Timers,
    C: Clock,
    I: Input,
    O: Output,
    // Optional peripherals:
    GB: OptionalPeripheral,
    OptTy<GB>: Gpio,
    GC: OptionalPeripheral,
    OptTy<GC>: Gpio,
{
    gpio: G,
    adc: A,
    pwm: P,
    timers: T,
    clock: C,
    input: I,
    output: O,
    gpio_bank_b: GB,
    gpio_bank_c: GC,
}

pub trait Bool { // TODO: seal
    const B: bool;
}
pub struct True; impl Bool for True { const B: bool = true; }
pub struct False; impl Bool for False { const B: bool = false; }
type T = True;
type F = False;

pub struct TypeTernary<Cond: Bool, IfTrue, IfFalse>(PhantomData<(Cond, IfTrue, IfFalse)>);
#[allow(type_alias_bounds)]
pub type Ternary<C: Bool, A, B> = <TypeTernary<C, A, B> as Eval>::Out;

pub trait Eval {
    type Out;
}

impl<A, B> Eval for TypeTernary<T, A, B> { type Out = A; }
impl<A, B> Eval for TypeTernary<F, A, B> { type Out = B; }

pub trait OptionalPeripheral { // TODO: sealed
    type Inner;

    // Requires that this be _statically_ known (PeripheralSet, that is).
    type Present: Bool;

    fn get(&self) -> Option<&Self::Inner> { None }
    fn get_mut(&mut self) -> Option<&mut Self::Inner> { None }
}
#[allow(type_alias_bounds)]
type OptTy<P: OptionalPeripheral> = <P as OptionalPeripheral>::Inner;
#[allow(type_alias_bounds)]
type OptPresent<P: OptionalPeripheral> = <P as OptionalPeripheral>::Present;

// We don't offer `OptionalPeripheral for G: Gpio`, etc. here because
// coherence keeps us from offering it for all of the peripheral traits
// i.e.:
// ```ignore
// impl<A: Adc> OptionalPeripheral for A { ... }
// ```
// would overlap with
// ```ignore
// impl<G: Gpio> OptionalPeripheral for G { ... }
// ```

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NotPresent<T>(PhantomData<T>);
impl<T> Default for NotPresent<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<T> OptionalPeripheral for NotPresent<T> {
    type Inner = T;
    type Present = False;
}

impl<T> Snapshot for NotPresent<T> {
    type Snap = ();
    type Err = Infallible;
    fn record(&self) -> Result<Self::Snap, Self::Err> { Ok(()) }
    fn restore(&mut self, _snap: Self::Snap) -> Result<(), Self::Err> { Ok(()) }
}

type NoGpio = NotPresent<gpio::MissingGpio>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, ambassador::Delegate)]
#[delegate(Snapshot)]
pub struct Present<T>(pub T);
impl<T> OptionalPeripheral for Present<T> {
    type Inner = T;
    type Present = True;
    fn get(&self) -> Option<&T> { None }
    fn get_mut(&mut self) -> Option<&mut T> { None }
}

impl<G, A, P, T, C, I, O, GB, GC> Default for PeripheralSet<G, A, P, T, C, I, O, GB, GC>
where
    G: Default + Gpio,
    A: Default + Adc,
    P: Default + Pwm,
    T: Default + Timers,
    C: Default + Clock,
    I: Default + Input,
    O: Default + Output,
    GB: Default + OptionalPeripheral,
    OptTy<GB>: Gpio,
    GC: Default + OptionalPeripheral,
    OptTy<GC>: Gpio,
{
    fn default() -> Self {
        Self {
            gpio: G::default(),
            adc: A::default(),
            pwm: P::default(),
            timers: T::default(),
            clock: C::default(),
            input: I::default(),
            output: O::default(),
            gpio_bank_b: GB::default(),
            gpio_bank_c: GC::default(),
        }
    }
}

impl<G, A, P, T, C, I, O> PeripheralSet<G, A, P, T, C, I, O>
where
    G: Gpio,
    A: Adc,
    P: Pwm,
    T: Timers,
    C: Clock,
    I: Input,
    O: Output,
{
    pub fn new(gpio: G, adc: A, pwm: P, timers: T, clock: C, input: I, output: O) -> Self {
        Self {
            gpio,
            adc,
            pwm,
            timers,
            clock,
            input,
            output,
            gpio_bank_b: Default::default(),
            gpio_bank_c: Default::default(),
        }
    }
}

impl<G, A, P, T, C, I, O, GB, GC> PeripheralSet<G, A, P, T, C, I, O, GB, GC>
where
    G: Gpio,
    A: Adc,
    P: Pwm,
    T: Timers,
    C: Clock,
    I: Input,
    O: Output,
    GB: OptionalPeripheral,
    OptTy<GB>: Gpio,
    GC: OptionalPeripheral,
    OptTy<GC>: Gpio,
{
    pub const HAS_GPIO_BANK_B: bool = <GB as OptionalPeripheral>::Present::B;
    pub const HAS_GPIO_BANK_C: bool = <GC as OptionalPeripheral>::Present::B;

    pub fn with_gpio_b<GpioBankB: Gpio>(self, gpio_bank_b: GpioBankB) -> PeripheralSet<G, A, P, T, C, I, O, Present<GpioBankB>, GC>
    {
        let Self {
            gpio,
            adc,
            pwm,
            timers,
            clock,
            input,
            output,
            gpio_bank_c,
            ..
        } = self;

        PeripheralSet {
            gpio, adc, pwm, timers, clock, input, output, gpio_bank_c,
            gpio_bank_b: Present(gpio_bank_b),
        }
    }

    pub fn with_gpio_c<GpioBankC: Gpio>(self, gpio_bank_c: GpioBankC) -> PeripheralSet<G, A, P, T, C, I, O, GB, Present<GpioBankC>>
    {
        let Self {
            gpio,
            adc,
            pwm,
            timers,
            clock,
            input,
            output,
            gpio_bank_b,
            ..
        } = self;

        PeripheralSet {
            gpio, adc, pwm, timers, clock, input, output, gpio_bank_b,
            gpio_bank_c: Present(gpio_bank_c),
        }
    }
}

impl<G, A, P, T, C, I, O, GB, GC> Peripherals for PeripheralSet<G, A, P, T, C, I, O, GB, GC>
where
    G: Gpio,
    A: Adc,
    P: Pwm,
    T: Timers,
    C: Clock,
    I: Input,
    O: Output,

    GB: OptionalPeripheral,
    OptTy<GB>: Gpio,
    GC: OptionalPeripheral,
    OptTy<GC>: Gpio,
{
    type Gpio = G;
    type Adc = A;
    type Pwm = P;
    type Timers = T;
    type Clock = C;
    type Input = I;
    type Output = O;

    type GpioB = OptTy<GB>;
    type GpioC = OptTy<GC>;

    fn get_gpio(&self) -> &G { &self.gpio }
    fn get_adc(&self) -> &A { &self.adc }
    fn get_pwm(&self) -> &P { &self.pwm }
    fn get_timers(&self) -> &T { &self.timers }
    fn get_clock(&self) -> &C { &self.clock }
    fn get_input(&self) -> &I { &self.input }
    fn get_output(&self) -> &O { &self.output }

    fn get_gpio_mut(&mut self) -> &mut Self::Gpio { &mut self.gpio }
    fn get_adc_mut(&mut self) -> &mut Self::Adc { &mut self.adc }
    fn get_pwm_mut(&mut self) -> &mut Self::Pwm { &mut self.pwm }
    fn get_timers_mut(&mut self) -> &mut Self::Timers { &mut self.timers }
    fn get_clock_mut(&mut self) -> &mut Self::Clock { &mut self.clock }
    fn get_input_mut(&mut self) -> &mut Self::Input { &mut self.input }
    fn get_output_mut(&mut self) -> &mut Self::Output { &mut self.output }

    fn get_gpio_bank_b(&self) -> Option<&Self::GpioB> {
        // To _enforce_ that it's fixed and not variable at runtime.
        <OptPresent<GB> as Bool>::B.then(|| self.gpio_bank_b.get().unwrap())
    }

    fn get_gpio_bank_b_mut(&mut self) -> Option<&mut Self::GpioB> {
        <OptPresent<GB> as Bool>::B.then(|| self.gpio_bank_b.get_mut().unwrap())
    }

    fn get_gpio_bank_c(&self) -> Option<&Self::GpioC> {
        <OptPresent<GC> as Bool>::B.then(|| self.gpio_bank_c.get().unwrap())
    }

    fn get_gpio_bank_c_mut(&mut self) -> Option<&mut Self::GpioC> {
        <OptPresent<GC> as Bool>::B.then(|| self.gpio_bank_c.get_mut().unwrap())
    }
}

use crate::control::{Snapshot, SnapshotError};

// /// where
// /// OptTy<OptPeri>: Gpio + Snapshot
// pub type SnapDataOr<OptPeri: OptionalPeripheral, Default = ()> = TypeTernary<
//     OptPresent<OptPeri>,
//     <OptTy<OptPeri> as Snapshot>::Snap,
//     Default,
// >;

// pub type GetSnapDataOr<OptPeri: OptionalPeripheral, Default = ()> = <
//     SnapDataOr<OptPeri, Default>
//     as
//     Eval
// >::Out;

impl<'p, G, A, P, T, C, I, O, GB, GC> Snapshot for PeripheralSet<G, A, P, T, C, I, O, GB, GC>
where
    G: Snapshot + Gpio,
    A: Snapshot + Adc,
    P: Snapshot + Pwm,
    T: Snapshot + Timers,
    C: Snapshot + Clock,
    I: Snapshot + Input,
    O: Snapshot + Output,

    GB: Snapshot + OptionalPeripheral,
    OptTy<GB>: Gpio,
    GC: Snapshot + OptionalPeripheral,
    OptTy<GC>: Gpio,

    // SnapDataOr<GB>: Eval,
    // SnapDataOr<GC>: Eval,

    // This shouldn't be needed since, in order to impl Snapshot your Err type has to
    // implement Into<SnapshotError>.
    // SnapshotError: From<<G as Snapshot>::Err>,
    // SnapshotError: From<<A as Snapshot>::Err>,
    // SnapshotError: From<<P as Snapshot>::Err>,
    // SnapshotError: From<<T as Snapshot>::Err>,
    // SnapshotError: From<<C as Snapshot>::Err>,
    // SnapshotError: From<<I as Snapshot>::Err>,
    // SnapshotError: From<<O as Snapshot>::Err>,
{
    type Snap = (
        <G as Snapshot>::Snap,
        <A as Snapshot>::Snap,
        <P as Snapshot>::Snap,
        <T as Snapshot>::Snap,
        <C as Snapshot>::Snap,
        <I as Snapshot>::Snap,
        <O as Snapshot>::Snap,
        // GetSnapDataOr<GB>,
        // GetSnapDataOr<GC>,
        <GB as Snapshot>::Snap,
        <GC as Snapshot>::Snap,
    );

    type Err = SnapshotError; // TODO: report which thing failed? make it part of the SnapshotError type?

    fn record(&self) -> Result<Self::Snap, Self::Err> {
        Ok((
            self.gpio.record().map_err(Into::into)?,
            self.adc.record().map_err(Into::into)?,
            self.pwm.record().map_err(Into::into)?,
            self.timers.record().map_err(Into::into)?,
            self.clock.record().map_err(Into::into)?,
            self.input.record().map_err(Into::into)?,
            self.output.record().map_err(Into::into)?,

            self.gpio_bank_b.record().map_err(Into::into)?,
            self.gpio_bank_c.record().map_err(Into::into)?,
        ))
    }

    fn restore(&mut self, snap: Self::Snap) -> Result<(), Self::Err> {
        let (g, a, p, t, c, i, o, gb, gc) = snap;

        self.gpio.restore(g).map_err(Into::into)?;
        self.adc.restore(a).map_err(Into::into)?;
        self.pwm.restore(p).map_err(Into::into)?;
        self.timers.restore(t).map_err(Into::into)?;
        self.clock.restore(c).map_err(Into::into)?;
        self.input.restore(i).map_err(Into::into)?;
        self.output.restore(o).map_err(Into::into)?;

        self.gpio_bank_b.restore(gb).map_err(Into::into)?;
        self.gpio_bank_c.restore(gc).map_err(Into::into)?;

        Ok(())
    }
}
