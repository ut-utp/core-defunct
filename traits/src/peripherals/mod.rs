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

use core::marker::PhantomData;


// the reason this trait exists is to accomodate implementations that wish to
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
pub trait Peripherals<'a> {
    type Gpio: Gpio<'a>;
    type Adc: Adc;
    type Pwm: Pwm;
    type Timers: Timers<'a>;
    type Clock: Clock;
    type Input: Input<'a>;
    type Output: Output<'a>;

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

pub struct PeripheralSet<'int, G, A, P, T, C, I, O>
where
    G: Gpio<'int>,
    A: Adc,
    P: Pwm,
    T: Timers<'int>,
    C: Clock,
    I: Input<'int>,
    O: Output<'int>,
{
    gpio: G,
    adc: A,
    pwm: P,
    timers: T,
    clock: C,
    input: I,
    output: O,
    _marker: PhantomData<&'int ()>,
}

impl<'p, G, A, P, T, C, I, O> Default for PeripheralSet<'p, G, A, P, T, C, I, O/*, G, A, P, T, C, I, O*/>
where
    G: Default + Gpio<'p>,
    A: Default + Adc,
    P: Default + Pwm,
    T: Default + Timers<'p>,
    C: Default + Clock,
    I: Default + Input<'p>,
    O: Default + Output<'p>,
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
            _marker: PhantomData,
        }
    }
}

impl<'p, G, A, P, T, C, I, O> PeripheralSet<'p, G, A, P, T, C, I, O>
where
    G: Gpio<'p>,
    A: Adc,
    P: Pwm,
    T: Timers<'p>,
    C: Clock,
    I: Input<'p>,
    O: Output<'p>,
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
            _marker: PhantomData,
        }
    }
}

impl<'p, G, A, P, T, C, I, O> Peripherals<'p> for PeripheralSet<'p, G, A, P, T, C, I, O>
where
    G: Gpio<'p>,
    A: Adc,
    P: Pwm,
    T: Timers<'p>,
    C: Clock,
    I: Input<'p>,
    O: Output<'p>,
{
    type Gpio = G;
    type Adc = A;
    type Pwm = P;
    type Timers = T;
    type Clock = C;
    type Input = I;
    type Output = O;

    fn get_gpio(&self) -> &G {
        &self.gpio
    }

    fn get_adc(&self) -> &A {
        &self.adc
    }

    fn get_pwm(&self) -> &P {
        &self.pwm
    }

    fn get_timers(&self) -> &T {
        &self.timers
    }

    fn get_clock(&self) -> &C {
        &self.clock
    }

    fn get_input(&self) -> &I {
        &self.input
    }

    fn get_output(&self) -> &O {
        &self.output
    }

    fn get_gpio_mut(&mut self) -> &mut Self::Gpio {
        &mut self.gpio
    }

    fn get_adc_mut(&mut self) -> &mut Self::Adc {
        &mut self.adc
    }

    fn get_pwm_mut(&mut self) -> &mut Self::Pwm {
        &mut self.pwm
    }

    fn get_timers_mut(&mut self) -> &mut Self::Timers {
        &mut self.timers
    }

    fn get_clock_mut(&mut self) -> &mut Self::Clock {
        &mut self.clock
    }

    fn get_input_mut(&mut self) -> &mut Self::Input {
        &mut self.input
    }

    fn get_output_mut(&mut self) -> &mut Self::Output {
        &mut self.output
    }
}

use crate::control::{Snapshot, SnapshotError};

impl<'p, G, A, P, T, C, I, O> Snapshot for PeripheralSet<'p, G, A, P, T, C, I, O>
where
    G: Snapshot + Gpio<'p>,
    A: Snapshot + Adc,
    P: Snapshot + Pwm,
    T: Snapshot + Timers<'p>,
    C: Snapshot + Clock,
    I: Snapshot + Input<'p>,
    O: Snapshot + Output<'p>,

    // This shouldn't be needed since, in order to impl Snapshot your Err type has to
    // implement Into<SnapshotError>.
    SnapshotError: From<<G as Snapshot>::Err>,
    SnapshotError: From<<A as Snapshot>::Err>,
    SnapshotError: From<<P as Snapshot>::Err>,
    SnapshotError: From<<T as Snapshot>::Err>,
    SnapshotError: From<<C as Snapshot>::Err>,
    SnapshotError: From<<I as Snapshot>::Err>,
    SnapshotError: From<<O as Snapshot>::Err>,
{
    type Snap = (
        <G as Snapshot>::Snap,
        <A as Snapshot>::Snap,
        <P as Snapshot>::Snap,
        <T as Snapshot>::Snap,
        <C as Snapshot>::Snap,
        <I as Snapshot>::Snap,
        <O as Snapshot>::Snap,
    );

    type Err = SnapshotError; // TODO: report which thing failed? make it part of the SnapshotError type?

    fn record(&self) -> Result<Self::Snap, Self::Err> {
        Ok((
            self.gpio.record()?,
            self.adc.record()?,
            self.pwm.record()?,
            self.timers.record()?,
            self.clock.record()?,
            self.input.record()?,
            self.output.record()?,
        ))
    }

    fn restore(&mut self, snap: Self::Snap) -> Result<(), Self::Err> {
        let (g, a, p, t, c, i, o) = snap;

        self.gpio.restore(g)?;
        self.adc.restore(a)?;
        self.pwm.restore(p)?;
        self.timers.restore(t)?;
        self.clock.restore(c)?;
        self.input.restore(i)?;
        self.output.restore(o)?;

        Ok(())
    }
}
