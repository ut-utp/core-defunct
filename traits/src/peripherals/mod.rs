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

// TODO: move to each module, introduce a type alias here for `PeripheralStubs`.
pub mod stubs;

use core::marker::PhantomData;

// TODO: do we still need this _trait_ to exist (as a super trait)?
pub trait Peripherals<'int>:
    Gpio<'int> + Adc + Pwm + Timers<'int> + Clock + Input<'int> + Output<'int>
{
    fn init(&mut self);
}
// no, i think we should just have:
/*

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
pub trait Peripherals {
    type Gpio: Gpio;
    type Adc: Adc;
    type Pwm: Pwm;
    type Timers: Timers;
    type Clock: Clock;
    type Input: Input;
    type Output: Output;

    fn get_gpio(&self) -> &Self::Gpio;
    fn get_gpio_mut(&mut self) -> &mut Self::Gpio;

    fn get_adc(&self) -> &Self::Adc;
    fn get_adc_mut(&mut self) -> &mut Self::Adc;

    fn get_pwm(&self) -> &Self::Pwm;
    fn get_pwm_mut(&mut self) -> &mut Self::Pwm;

    fn get_timers(&self) -> &Self::Timers;
    fn get_timers_mut(&mut self) -> &mut Self::Timers;

    fn get_input(&self) -> &Self::Input;
    fn get_input_mut(&mut self) -> &mut Self::Input;

    fn get_output(&self) -> &Self::Output;
    fn get_output_mut(&mut self) -> &mut Self::Output;
}
 */


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

pub struct PeripheralSet<'int, G, A, P, T, C, I, O/*, GW, AW, PW, TW, CW, IW, OW*/>
where
    G: Gpio<'int>,
    A: Adc,
    P: Pwm,
    T: Timers<'int>,
    C: Clock,
    I: Input<'int>,
    O: Output<'int>,
    // GW: 'p + DerefOrOwned<G>,
    // AW: 'p + DerefOrOwned<A>,
    // PW: 'p + DerefOrOwned<P>,
    // TW: 'p + DerefOrOwned<T>,
    // CW: 'p + DerefOrOwned<C>,
    // IW: 'p + DerefOrOwned<I>,
    // OW: 'p + DerefOrOwned<O>,
{
    // gpio: GW,
    // adc: AW,
    // pwm: PW,
    // timers: TW,
    // clock: CW,
    // input: IW,
    // output: OW,
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

impl<'p, G, A, P, T, C, I, O/*, GW, AW, PW, TW, CW, IW, OW*/> PeripheralSet<'p, G, A, P, T, C, I, O/*, GW, AW, PW, TW, CW, IW, OW*/>
where
    G: Gpio<'p>,
    A: Adc,
    P: Pwm,
    T: Timers<'p>,
    C: Clock,
    I: Input<'p>,
    O: Output<'p>,
    // GW: 'p + DerefOrOwned<G>,
    // AW: 'p + DerefOrOwned<A>,
    // PW: 'p + DerefOrOwned<P>,
    // TW: 'p + DerefOrOwned<T>,
    // CW: 'p + DerefOrOwned<C>,
    // IW: 'p + DerefOrOwned<I>,
    // OW: 'p + DerefOrOwned<O>,
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

    pub fn get_gpio(&self) -> &G {
        &self.gpio
    }

    pub fn get_adc(&self) -> &A {
        &self.adc
    }

    pub fn get_pwm(&self) -> &P {
        &self.pwm
    }

    pub fn get_timers(&self) -> &T {
        &self.timers
    }

    pub fn get_clock(&self) -> &C {
        &self.clock
}

    pub fn get_input(&self) -> &I {
        &self.input
    }

    pub fn get_output(&self) -> &O {
        &self.output
    }
}

// enum WrapperType {
//     Dual,
//     Mutless,
// }

// // TODO: type to lock on each access that implements
// trait RunInContext<Wrapped> {
//     const TYPE: WrapperType;

//     fn with_ref<R, F: FnOnce(&Wrapped) -> R>(&self, func: F) -> R;
//     fn with_mut<R, F: FnOnce(&mut Wrapped) -> R>(&mut self, func: F) -> R;
// }

// impl<T, P: AsRef<T> + AsMut<T>> RunInContext<T> for P {
//     const TYPE: WrapperType = WrapperType::Dual;

//     fn with_ref<R, F: FnOnce(&T) -> R>(&self, func: F) -> R { func(self.as_ref()) }
//     fn with_mut<R, F: FnOnce(&mut T) -> R>(&mut self, func: F) -> R { func(self.as_mut()) }
// }

// impl<T, I: RunInContext<T>> RunInContext<T> for Arc<I> {
//     fn with_ref<R, F: FnOnce(&T) -> R>(&self, func: F) -> R {
//         func()
//     }

//     fn with_mut<R, F: FnOnce(&T) -> R>(&mut self, func: F) -> F {

//     }
// }

// struct LockOnAccess<T>(T);

// struct BorrowOnAccess<T>(T);

#[doc(hidden)]
#[macro_export]
macro_rules! peripheral_trait {
    ($nom:ident, $(#[$attr:meta])* pub trait $trait:ident $(<$lifetime:lifetime>)? $(: $bound:ident )? { $($rest:tt)* }) => {
        $(#[$attr])*
        pub trait $trait $(<$lifetime>)? where Self: $($bound)? { $($rest)* }

        // $crate::deref_impl!($trait$(<$lifetime>)? $(| $lifetime |)?, { $($rest)* });
        // $crate::borrow_impl!($trait$(<$lifetime>)? $(| $lifetime |)?, { $($rest)* });
        $crate::peripheral_set_impl!($trait$(<$lifetime>)? $(| $lifetime |)?, { $crate::func_sig!($nom, $($rest)*); });
        // $crate::peripheral_deref_set_impl!($trait$(<$lifetime>)? $(| $lifetime |)?, { $crate::func_sig!($nom, $($rest)*); });
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! deref_impl {
    ($trait:path $(| $lifetime:lifetime |)?, { $($rest:tt)* }) => {
        #[allow(unnecessary_qualification)]
        impl<$($lifetime,)? I, T: Default + core::ops::Deref<Target = I> + core::ops::DerefMut> $trait for T
        where
            I: $trait,
        { $crate::func_sig!(+(*), $($rest)*); }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! borrow_impl {
    ($trait:path $(| $lifetime:lifetime |)?, { $($rest:tt)* }) => {
        #[allow(unnecessary_qualification)]
        impl<$($lifetime,)? I, T: Default + core::ops::Deref<Target = I> + core::ops::DerefMut + core::borrow::Borrow<I> + core::borrow::BorrowMut<I>> $trait for T
        where
            I: $trait,
        { $crate::func_sig!(%(borrow, borrow_mut), $($rest)*); }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! peripheral_set_impl {
    ($trait:ty $(| $lifetime:lifetime |)?, { $($rest:tt)* }) => {
        impl<$($lifetime,)? 'p, G, A, P, T, C, I, O> $trait for $crate::peripherals::PeripheralSet<'p, G, A, P, T, C, I, O/*, G, A, P, T, C, I, O*/>
        where
            $($lifetime: 'p,)?
            G: $crate::peripherals::gpio::Gpio<'p>,
            A: $crate::peripherals::adc::Adc,
            P: $crate::peripherals::pwm::Pwm,
            T: $crate::peripherals::timers::Timers<'p>,
            C: $crate::peripherals::clock::Clock,
            I: $crate::peripherals::input::Input<'p>,
            O: $crate::peripherals::output::Output<'p>,
        { $($rest)* }
    };
}

// #[doc(hidden)]
// #[macro_export]
// macro_rules! peripheral_deref_set_impl {
//     ($trait:ty $(| $lifetime:lifetime |)?, { $($rest:tt)* }) => {
//         impl<$($lifetime,)? 'p, G, A, P, T, C, I, O, GInner, AInner, PInner, TInner, CInner, IInner, OInner> $trait for $crate::peripherals::PeripheralSet<'p, G, A, P, T, C, I, O>
//         where
//             $($lifetime: 'p,)?
//             G: 'p + $crate::peripherals::gpio::Gpio<'p>,
//             A: 'p + $crate::peripherals::adc::Adc,
//             P: 'p + $crate::peripherals::pwm::Pwm,
//             T: 'p + $crate::peripherals::timers::Timers<'p>,
//             C: 'p + $crate::peripherals::clock::Clock,
//             I: 'p + $crate::peripherals::input::Input<'p>,
//             O: 'p + $crate::peripherals::output::Output<'p>,
//             GInner: 'p + Deref<
//         { $($rest)* }
//     };
// }

#[doc(hidden)]
#[macro_export]
macro_rules! func_sig {
    // [No block + Ret] Our ideal form: specified return type, no block:
    // (none)
    ($(+($indir:tt))? $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident($($idents:ident : $types:ty),*) -> $ret:ty; $($rest:tt)*) => {
        #[inline] fn $fn_name($($idents : $types),*) -> $ret { compile_error!("trait functions not supported yet!") }
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, $($rest)*);
    };
    // (self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(self$(,)? $($idents:ident : $types:ty),*) -> $ret:ty; $($rest:tt)*) => {
        #[inline] fn $fn_name(self, $($idents : $types),*) -> $ret { ($($indir$indir)?self)$(.$nom)?$(.$i_im())?.$fn_name($($idents),*) }
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, $($rest)*);
    };
    // (mut self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(mut self$(,)? $($idents:ident : $types:ty),*) -> $ret:ty; $($rest:tt)*) => {
        #[inline] fn $fn_name(mut self, $($idents : $types),*) -> $ret { ($($indir$indir)?self)$(.$nom)?$(.$i_mut())?.$fn_name($($idents),*) }
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, $($rest)*);
    };
    // (&self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(&self$(,)? $($idents:ident : $types:ty),*) -> $ret:ty; $($rest:tt)*) => {
        #[inline] fn $fn_name(&self, $($idents : $types),*) -> $ret { ($($indir$indir)?self)$(.$nom)?$(.$i_im())?.$fn_name($($idents),*) }
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, $($rest)*);
    };
    // (&mut self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(&mut self$(,)? $($idents:ident : $types:ty),*) -> $ret:ty; $($rest:tt)*) => {
        #[inline] fn $fn_name(&mut self, $($idents : $types),*) -> $ret { ($($indir$indir)?self)$(.$nom)?$(.$i_mut())?.$fn_name($($idents),*) }
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, $($rest)*);
    };


    // [Block + Ret] Ditch blocks if you've got them:
    // (none)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident($($idents:ident : $types:ty),*) -> $ret:ty $block:block $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name($($idents : $types),*) -> $ret; $($rest)*); };
    // (self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(self$(,)? $($self:expr,)? $($idents:ident : $types:ty),*) -> $ret:ty $block:block $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name(self, $($idents : $types),*) -> $ret; $($rest)*); };
    // (mut self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(mut self$(,)? $($idents:ident : $types:ty),*) -> $ret:ty $block:block $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name(mut self, $($idents : $types),*) -> $ret; $($rest)*); };
    // (&self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(&self$(,)? $($idents:ident : $types:ty),*) -> $ret:ty $block:block $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name(&self, $($idents : $types),*) -> $ret; $($rest)*); };
    // (&mut self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(&mut self$(,)? $($idents:ident : $types:ty),*) -> $ret:ty $block:block $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name(&mut self, $($idents : $types),*) -> $ret; $($rest)*); };


    // [No Block + No Ret] Add in return types if they're not specified:
    // (none)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident($($idents:ident : $types:ty),*); $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name($($idents : $types),*) -> (); $($rest)*); };
    // (self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(self$(,)? $($idents:ident : $types:ty),*); $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name(self, $($idents : $types),*) -> (); $($rest)*); };
    // (mut self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(mut self$(,)? $($idents:ident : $types:ty),*); $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name(mut self, $($idents : $types),*) -> (); $($rest)*); };
    // (&self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(&self$(,)? $($idents:ident : $types:ty),*); $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name(&self, $($idents : $types),*) -> (); $($rest)*); };
    // (&mut self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(&mut self$(,)? $($idents:ident : $types:ty),*); $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name(&mut self, $($idents : $types),*) -> (); $($rest)*); };


    // [Block + No Ret] Strip blocks + add in return types:
    // (none)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident( $($idents:ident : $types:ty),*) $block:block $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name($($idents : $types),*) -> (); $($rest)*); };
    // (self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(self$(,)? $($idents:ident : $types:ty),*) $block:block $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name(self, $($idents : $types),*) -> (); $($rest)*); };
    // (mut self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(mut self$(,)? $($idents:ident : $types:ty),*) $block:block $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name(mut self, $($idents : $types),*) -> (); $($rest)*); };
    // (&self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(&self$(,)? $($idents:ident : $types:ty),*) $block:block $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name(&self $($idents : $types),*) -> (); $($rest)*); };
    // (&mut self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(&mut self$(,)? $($idents:ident : $types:ty),*) $block:block $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name(&mut self, $($idents : $types),*) -> (); $($rest)*); };


    // And, finally, the end:
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, ) => {};
}

impl<'p, G, A, P, T, C, I, O> Peripherals<'p> for PeripheralSet<'p, G, A, P, T, C, I, O/*, G, A, P, T, C, I, O*/>
where
    G: Gpio<'p>,
    A: Adc,
    P: Pwm,
    T: Timers<'p>,
    C: Clock,
    I: Input<'p>,
    O: Output<'p>,
{
    fn init(&mut self) {}
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
