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
// #[ambassador::delegatable_trait]
// pub trait Peripherals {
//     type Gpio: ?Sized + Gpio;
//     type Adc: ?Sized + Adc;
//     type Pwm: ?Sized + Pwm;
//     type Timers: ?Sized + Timers;
//     type Clock: ?Sized + Clock;
//     type Input: ?Sized + Input;
//     type Output: ?Sized + Output;

//     type GpioB: ?Sized + Gpio/*  = gpio::MissingGpio */;
//     type GpioC: ?Sized + Gpio/*  = gpio::MissingGpio */;

//     fn get_gpio(&self) -> &Self::Gpio;
//     fn get_gpio_mut(&mut self) -> &mut Self::Gpio;

//     fn get_adc(&self) -> &Self::Adc;
//     fn get_adc_mut(&mut self) -> &mut Self::Adc;

//     fn get_pwm(&self) -> &Self::Pwm;
//     fn get_pwm_mut(&mut self) -> &mut Self::Pwm;

//     fn get_timers(&self) -> &Self::Timers;
//     fn get_timers_mut(&mut self) -> &mut Self::Timers;

//     fn get_clock(&self) -> &Self::Clock;
//     fn get_clock_mut(&mut self) -> &mut Self::Clock;

//     fn get_input(&self) -> &Self::Input;
//     fn get_input_mut(&mut self) -> &mut Self::Input;

//     fn get_output(&self) -> &Self::Output;
//     fn get_output_mut(&mut self) -> &mut Self::Output;

//     // optional peripherals:
//     fn get_gpio_bank_b(&self) -> Option<&Self::GpioB> { None }
//     fn get_gpio_bank_b_mut(&mut self) -> Option<&mut Self::GpioB> { None }

//     fn get_gpio_bank_c(&self) -> Option<&Self::GpioC> { None }
//     fn get_gpio_bank_c_mut(&mut self) -> Option<&mut Self::GpioC> { None }
// }

// a macro to keep the peripheral impls in sync between the `Peripherals` trait and the `PeripheralSet`/`PeripheralsWrapper` types
// (and to handle the convention around optional peripherals)
//
// TODO: document the convention around optional peripherals here
// also: we assume all traits have `ambassador::delegatable_trait` on them, etc.
macro_rules! peripherals {
    (
        $(#[$attrs:meta])*
        pub trait $nom:ident = {
            required: {
                $(
                    $(#[$req_peri_attr:meta])*
                    $req_peri_name:ident : $req_peri_trait:ident ($req_ty_short:ident)
                ),* $(,)?
            },

            optional: {
                $(
                    $(#[$opt_peri_attr:meta])*
                    $opt_peri_name:ident : $opt_peri_trait:ident ($opt_ty_short:ident) as $opt_peri_ty_param_name:ident
                        (with
                            $(using(
                                ($opt_peri_stub:ty) in $opt_peri_placeholder:ident wrapped as $opt_peri_alias:ident
                            ):)?
                            default($opt_peri_def:ty)
                        )
                ),* $(,)?
            }$(,)?
        } with {
            $(#[$set_attrs:meta])*
            set = struct $set_ty:ident<...>;

            $(#[$wrapper_attrs:meta])*
            wrapper = struct $wrapper_ty:ident<_>;
        }
    ) => {
        /* Define the peripheral trait */

        $(#[$attrs])*
        pub trait $nom {
            // Required peripheral types:
            $(
                $(#[$req_peri_attr])*
                type $req_peri_trait: ?Sized + $req_peri_trait;
            )*

            // Optional peripheral types:
            $(
                $(#[$opt_peri_attr])*
                #[doc = "\n\n"]
                #[doc = "Use [`"]
                #[doc = core::stringify!($opt_peri_def)]
                #[doc = "`] if you don't want to provide an actual [`"]
                #[doc = core::stringify!($opt_peri_trait)]
                #[doc = "`] implementation!\n"]
                type $opt_peri_ty_param_name: ?Sized + $opt_peri_trait /* = $opt_peri_def */;
            )*

            // Required peripheral getters:
            $(
                peripherals!(@getter_pair_def: ($req_peri_name) {
                    #[doc = "for the [`"]
                    #[doc = core::stringify!($req_peri_trait)]
                    #[doc = "`] peripheral."]
                } -> &Self::$req_peri_trait, &mut Self::$req_peri_trait = with self rest(;));
                // paste::paste! {
                //     #[doc = "Getter for the [`"]
                //     #[doc = core::stringify!($req_peri_trait)]
                //     #[doc = "`] peripheral."]
                //     fn [< get_ $req_peri_name >](&self) -> &Self::$req_peri_trait;

                //     #[doc = "_Mutable_ getter for the [`"]
                //     #[doc = core::stringify!($req_peri_trait)]
                //     #[doc = "`] peripheral."]
                //     fn [< get_ $req_peri_name _mut >](&mut self) -> &mut Self::$req_peri_trait;
                // }
            )*

            // Optional peripheral getters:
            $(
                peripherals!(@getter_pair_def: ($opt_peri_name) {
                    #[doc = " for [`"]
                        #[doc = core::stringify!($nom)]
                        #[doc = "::"]
                        #[doc = core::stringify!($opt_peri_ty_param_name)]
                    #[doc = "`] (the **optional** [`"]
                    #[doc = core::stringify!($opt_peri_trait)]
                    #[doc = "`] peripheral)."]
                    #[doc = "\n\n"]
                    #[doc = "Defaults to returning [`None`], remember to implement this if you \n"]
                    #[doc = "**don't** set [`"]
                        #[doc = core::stringify!($nom)]
                        #[doc = "::"]
                        #[doc = core::stringify!($opt_peri_ty_param_name)]
                    #[doc = "`] to [`"]
                    #[doc = core::stringify!($opt_peri_def)]
                    #[doc = "`]."]
                } -> Option<&Self::$opt_peri_ty_param_name>, Option<&mut Self::$opt_peri_ty_param_name> =
                    with self rest({ None })
                );

                // paste::paste! {
                //     // #[doc = "Getter
                //     #[doc = " for [`"]
                //         #[doc = core::stringify!($nom)]
                //         #[doc = "::"]
                //         #[doc = core::stringify!($opt_peri_ty_param_name)]
                //     #[doc = "`] (the **optional** [`"]
                //     #[doc = core::stringify!($opt_peri_trait)]
                //     #[doc = "`] peripheral)."]
                //     #[doc = "\n\n"]
                //     #[doc = "Defaults to returning [`None`], remember to implement this if you \n"]
                //     #[doc = "**don't** set [`"]
                //         #[doc = core::stringify!($nom)]
                //         #[doc = "::"]
                //         #[doc = core::stringify!($opt_peri_ty_param_name)]
                //     #[doc = "`] to [`"]
                //     #[doc = core::stringify!($opt_peri_def)]
                //     #[doc = "`]."]
                //     fn [< get_ $opt_peri_name >](&self) -> Option<&Self::$req_peri_trait> { None }

                //     #[doc = "_Mutable_ getter for the [`"]
                //         #[doc = core::stringify!($nom)]
                //         #[doc = "::"]
                //         #[doc = core::stringify!($opt_peri_ty_param_name)]
                //     #[doc = "`] peripheral."]
                //     fn [< get_ $opt_peri_name _mut >](&mut self) -> &mut Self::$req_peri_trait;
                // }
            )*
        }

        /* Define the optional peripheral placeholders + aliases: */
        $($(
            optional_peripheral_placeholder_from_stub!(
                trait: $opt_peri_trait,
                stub: $opt_peri_stub,
                placeholder: $opt_peri_placeholder,
                alias: $opt_peri_alias,
            );
        )?)*

        // set, defaults for optional ty_params
        // impl Peripherals for Set
        // delegate all traits to Set
        // impl builder for each optional ty param
        // assoc const for each optional ty
        // impl Snapshot

        /* Define the Peripheral Set: */
        peripherals!(@add_attrs:
            (
                // #[derive(ambassador::Delegate)]
                // $(
                //     #[delegate($req_peri_trait, target = core::stringify!($req_peri_name))]
                // )*
            ) to
            $(#[$set_attrs])*
            pub struct $set_ty<
                $($req_ty_short,)*
                $($opt_ty_short = $opt_peri_def,)*
            >
            where
                // Required peripheral bounds:
                $($req_ty_short: $req_peri_trait,)*

                // Optional peripheral bounds:
                $(
                    $opt_ty_short: OptionalPeripheral,
                    OptTy<$opt_ty_short>: $opt_peri_trait,
                )*
            {
                $(
                    #[doc = "An instance of a(n) [`"]
                    #[doc = core::stringify!($req_peri_trait)]
                    #[doc = "`] implementation (required)."]
                    #[doc = "\n\n"]
                    #[doc = "Corresponds to [`"]
                    #[doc = core::stringify!($nom)]
                    #[doc = "::"]
                    #[doc = core::stringify!($req_peri_trait)]
                    #[doc = "`]."]
                    $req_peri_name: $req_ty_short,
                )*

                $(
                    #[doc = "An instance of a(n) [`"]
                    #[doc = core::stringify!($opt_peri_trait)]
                    #[doc = "`] implementation (optional)."]
                    #[doc = "\n\n"]
                    #[doc = "Defaults to [`"]
                    #[doc = core::stringify!($opt_peri_def)]
                    #[doc = "`]."]
                    #[doc = "\n\n"]
                    #[doc = "Corresponds to [`"]
                    #[doc = core::stringify!($nom)]
                    #[doc = "::"]
                    #[doc = core::stringify!($opt_peri_ty_param_name)]
                    #[doc = "`]."]
                    $opt_peri_name: $opt_ty_short,
                )*
            }
        );

        /* Implement the Peripherals trait for the Peripherals Set: */
        impl<
            $($req_ty_short,)*
            $($opt_ty_short,)*
        > $nom for $set_ty<
            $($req_ty_short,)*
            $($opt_ty_short,)*
        >
        where
            $($req_ty_short: $req_peri_trait,)*
            $(
                $opt_ty_short: OptionalPeripheral,
                OptTy<$opt_ty_short>: $opt_peri_trait,
            )*
        {
            // Associated types:
            $( type $req_peri_trait = $req_ty_short; )*
            $( type $opt_peri_ty_param_name = OptTy<$opt_ty_short>; )*

            // Getters:
            $(
                peripherals!(@getter_pair_def: ($req_peri_name) {
                        #[inline(always)]
                    } -> &Self::$req_peri_trait, &mut Self::$req_peri_trait =
                    with self
                    cond[
                        self: (&self.$req_peri_name)
                        mut self: (&mut self.$req_peri_name)
                    ]
                );
            )*
            $(
                peripherals!(@getter_pair_def: ($opt_peri_name) {
                        #[inline(always)]
                    } -> Option<&Self::$opt_peri_ty_param_name>, Option<&mut Self::$opt_peri_ty_param_name> =
                    with self
                    cond[
                        // To _enforce_ that it's fixed and not variable at runtime:
                        self: (
                            <OptPresent<$opt_ty_short> as Bool>::B.then(|| self.$opt_peri_name.get().unwrap())
                        )
                        mut self: (
                            <OptPresent<$opt_ty_short> as Bool>::B.then(|| self.$opt_peri_name.get_mut().unwrap())
                        )
                    ]
                );
            )*
        }

        /* Builders for the Peripherals Set: */
        impl<$($req_ty_short,)*> $set_ty<$($req_ty_short,)*>
        where
            $( $req_ty_short: $req_peri_trait, )*
        {
            // TODO: docs
            pub fn new(
                $($req_peri_name: $req_ty_short,)*
            ) -> Self {
                Self {
                    $($req_peri_name,)*
                    $($opt_peri_name: Default::default(),)*
                }
            }
        }

        impl<
            $($req_ty_short,)*
            $($opt_ty_short,)*
        > $set_ty<
            $($req_ty_short,)*
            $($opt_ty_short,)*
        >
        where
            $($req_ty_short: $req_peri_trait,)*
            $(
                $opt_ty_short: OptionalPeripheral,
                OptTy<$opt_ty_short>: $opt_peri_trait,
            )*
        {
            // Associated consts for optional peripherals:
            $(
                // todo: the only reason this is not an associated const instead of
                // an associated const fn is because of the case (snake case instead of
                // screaming snake case)
                paste::paste! {
                    #[doc = "Whether or not this [`"]
                    #[doc = core::stringify!($set_ty)]
                    #[doc = "`] type has an implementation for [`"]
                        #[doc = core::stringify!($nom)]
                        #[doc = "::"]
                        #[doc = core::stringify!($opt_peri_ty_param_name)]
                    #[doc = "`]."]
                    pub const fn [< has_ $opt_peri_name >]() -> bool {
                        <$opt_ty_short as OptionalPeripheral>::Present::B
                    }
                }
            )*

            peripherals!{@builder_method_for_each_optional_peripheral
                peri_trait: $nom
                on: $set_ty
                required: ($(
                    ( $req_peri_name: $req_peri_trait ($req_ty_short) )
                )*)
                optional_finished: (

                )
                optional_pending: ($(
                    ( $opt_peri_name: $opt_peri_trait ($opt_ty_short) as $opt_peri_ty_param_name )
                )*)
            }
        }


        $(#[$wrapper_attrs])*
        #[repr(transparent)]
        // TODO: docs
        pub struct $wrapper_ty<P: ?Sized + $nom>(P);

        impl<P: ?Sized + $nom> $wrapper_ty<P> {
            // TODO: docs
            pub fn get_inner_peripherals(&self) -> &P { &self.0 }
            // TODO: docs
            pub fn get_inner_peripherals_mut(&mut self) -> &mut P { &mut self.0 }
        }

        impl<P: ?Sized + $nom> $wrapper_ty<P> {

        }

        // impl Peripherals for Wrapper
        // delegate all traits to Wrapper

        // impl traits on snapshotusingclone
        // impl traits on the `GetRwLock`/`GetMutex` things
        //
        // automatically do this for the required peripheral traits; add a
        // static assertion checking that this has been done for the optional
        // peripheral traits!
    };

    (@getter_pair_def: ($name:ident) {
        $(#[$attrs:meta])*
    } -> $ret_ty:ty, $ret_ty_mut:ty =
        with $self:ident
        $(cond[self : ($($s:tt)*) mut self: ($($ms:tt)*)])?
        $(rest($($rest:tt)*))?
    ) => {
        paste::paste! {
            #[doc = "Getter "]
            $(#[$attrs])*
            fn [< get_ $name >](&$self) -> $ret_ty $({ $($s)* })? $($($rest)*)?

            #[doc = "_Mutable_ getter "]
            $(#[$attrs])*
            fn [< get_ $name _mut >](&mut $self) -> $ret_ty_mut $({ $($ms)* })? $($($rest)*)?
        }
    };

    // Our silly way of "forcing" macro_rules expansion before proc macro
    // expansion.
    (@add_attrs: ($(#[$attr:meta])*) to $($tt:tt)*) => {
        $(#[$attr])*
        $($tt)*
    };

    /*  */
    (@builder_method_for_each_optional_peripheral
        peri_trait: $nom:ident
        on: $set_ty:ident
        required: ($(
            ( $req_peri_name:ident : $_req_peri_trait:ident ($req_ty_short:ident) )
        )*)
        optional_finished: ($(
            ( $opt_peri_name_f:ident : $opt_peri_trait_f:ident ($opt_ty_short_f:ident) as $opt_peri_ty_param_name_f:ident )
        )*)
        optional_pending: (
            ( $opt_peri_name_first:ident : $opt_peri_trait_first:ident ($opt_ty_short_first:ident) as $opt_peri_ty_param_name_first:ident )
            $(
                ( $opt_peri_name_rest:ident : $opt_peri_trait_rest:ident ($opt_ty_short_rest:ident) as $opt_peri_ty_param_name_rest:ident )
            )*
        )
    ) => {
        paste::paste! {
            #[doc = "Sets [`"]
            #[doc = core::stringify!($set_ty)]
            #[doc = "`]'s instance for the optional peripheral [`"]
                #[doc = core::stringify!($nom)]
                #[doc = "::"]
                #[doc = core::stringify!($opt_peri_ty_param_name_first)]
            #[doc = "`] to an instance of an actual implementation."]
            pub fn [< with_ $opt_peri_name_first >]<
                $opt_peri_ty_param_name_first: $opt_peri_trait_first
            >(self, $opt_peri_name_first: $opt_peri_ty_param_name_first) -> $set_ty<
                $($req_ty_short,)*
                $($opt_ty_short_f,)*
                Present<$opt_peri_ty_param_name_first>,
                $($opt_ty_short_rest,)*
            > {
                let Self {
                    $($req_peri_name,)*
                    $($opt_peri_name_f,)*

                    $($opt_peri_name_rest,)*

                    ..
                } = self;

                $set_ty {
                    $($req_peri_name,)*
                    $($opt_peri_name_f,)*
                    $opt_peri_name_first: Present($opt_peri_name_first),
                    $($opt_peri_name_rest,)*
                }
            }
        }

        peripherals!{ @builder_method_for_each_optional_peripheral
            peri_trait: $nom
            on: $set_ty
            required: ($(
                ( $req_peri_name : $_req_peri_trait ($req_ty_short) )
            )*)
            optional_finished: (
                $(
                    ( $opt_peri_name_f : $opt_peri_trait_f ($opt_ty_short_f) as $opt_peri_ty_param_name_f )
                )*
                ( $opt_peri_name_first : $opt_peri_trait_first ($opt_ty_short_first) as $opt_peri_ty_param_name_first )
            )
            optional_pending: ( $(
                ( $opt_peri_name_rest : $opt_peri_trait_rest ($opt_ty_short_rest) as $opt_peri_ty_param_name_rest )
            )* )
        }
    };

    (@builder_method_for_each_optional_peripheral
        peri_trait: $nom:ident
        on: $set_ty:ident
        required: ($(
            ( $req_peri_name:ident : $req_peri_trait:ident ($req_ty_short:ident) )
        )*)
        optional_finished: ($(
            ( $a:ident : $b:ident ($c:ident) as $d:ident )
        )*)
        optional_pending: ( )
    ) => {
        /* fin */
    };
}

// A consequence of our optional peripheral convention is that even when an
// optional peripheral is _not_ present, we still need to provide a type that
// implements that peripheral's trait.
//
// This macro defines such peripheral trait implementations concisely.
//
// Altogether this macro:
//   - defines a _hidden_ implementation of the given peripheral trait by
//     pretending to shell out to the peripheral trait's stub
//   - creates a type alias of the implementation with `NotPresent`
//   - implements `Snapshot` for the placeholder
macro_rules! optional_peripheral_placeholder_from_stub {
    (
        trait: $t:ident,
        stub: $s:ty,
        placeholder: $p:ident,
        alias: $a:ident $(,)?
    ) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[derive(serde::Serialize, serde::Deserialize)]
        #[doc(hidden)]
        pub enum $p { /* unconstructible by design */}

        #[ambassador::delegate_to_methods]
        #[delegate($t, target_ref = "get", target_mut = "get_mut")]
        impl $p {
            fn get(&self) -> &$s { unreachable!() }
            fn get_mut(&mut self) -> &mut $s { unreachable!() }
        }

        #[doc = ""] // TODO!
        pub type $a = NotPresent<$p>;

        impl Snapshot for $p {
            type Snap = ();
            type Err = Infallible;
            fn record(&self) -> Result<Self::Snap, Self::Err> { Ok(()) }
            fn restore(&mut self, _snap: Self::Snap) -> Result<(), Self::Err> {
                Ok(())
            }
        }
    };
}

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
        gpio_bank_b: Gpio (GB) as GpioB
            (with using((stubs::GpioStub) in MissingGpio wrapped as NoGpio): default(NoGpio)),
        gpio_bank_c: Gpio (GC) as GpioC
            (with default(NoGpio)),
    },
} with {
    /// TODO!
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[derive(serde::Serialize, serde::Deserialize)]
    set = struct PeripheralSet<...>;

    /// TODO!
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

// TODO: docs explaining that the utility here is getting a type that
// has delegated impls of all the peripheral traits + `Peripherals` implemented!
// #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash, serde::Serialize, serde::Deserialize)]
// #[repr(transparent)]

// pub struct PeripheralsWrapper<P: ?Sized + Peripherals>(P);

// impl<P: ?Sized + Peripherals> PeripheralsWrapper<P> {
//     pub fn get_inner_peripherals(&self) -> &P { &self.0 }
//     pub fn get_inner_peripherals_mut(&mut self) -> &mut P { &mut self.0 }
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
impl<P: ?Sized + Peripherals> Peripherals for PeripheralsWrapper<P> {
    type Gpio = P::Gpio;
    type Adc = P::Adc;
    type Pwm = P::Pwm;
    type Timers = P::Timers;
    type Clock = P::Clock;
    type Input = P::Input;
    type Output = P::Output;

    type GpioB = P::GpioB;
    type GpioC = P::GpioC;

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

    /* todo: optional! */
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

// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, ambassador::Delegate)]
// #[derive(Default)]
// #[delegate(Gpio, target = "gpio")]
// #[delegate(Adc, target = "adc")]
// #[delegate(Pwm, target = "pwm")]
// #[delegate(Timers, target = "timers")]
// #[delegate(Clock, target = "clock")]
// #[delegate(Input, target = "input")]
// #[delegate(Output, target = "output")]
// pub struct PeripheralSet<G, A, P, T, C, I, O, GB = NoGpio, GC = NoGpio>
// where
//     G: Gpio,
//     A: Adc,
//     P: Pwm,
//     T: Timers,
//     C: Clock,
//     I: Input,
//     O: Output,
//     // Optional peripherals:
//     GB: OptionalPeripheral,
//     OptTy<GB>: Gpio,
//     GC: OptionalPeripheral,
//     OptTy<GC>: Gpio,
// {
//     gpio: G,
//     adc: A,
//     pwm: P,
//     timers: T,
//     clock: C,
//     input: I,
//     output: O,
//     gpio_bank_b: GB,
//     gpio_bank_c: GC,
// }

pub trait Bool: sealed::Sealed {
    const B: bool;
}
pub struct True; impl Bool for True { const B: bool = true; }
pub struct False; impl Bool for False { const B: bool = false; }

mod sealed {
    pub trait Sealed { }

    impl Sealed for super::True { }
    impl Sealed for super::False { }

    impl<T> Sealed for super::Present<T> {}
    impl<T: ?Sized> Sealed for super::NotPresent<T> {}
}

pub trait OptionalPeripheral: sealed::Sealed {
    type Inner: ?Sized;

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
#[derive(serde::Serialize, serde::Deserialize)]
pub struct NotPresent<T: ?Sized>(PhantomData<T>);
impl<T: ?Sized> Default for NotPresent<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<T: ?Sized> OptionalPeripheral for NotPresent<T> {
    type Inner = T;
    type Present = False;
}

impl<T: ?Sized> Snapshot for NotPresent<T> {
    type Snap = ();
    type Err = Infallible;
    fn record(&self) -> Result<Self::Snap, Self::Err> { Ok(()) }
    fn restore(&mut self, _snap: Self::Snap) -> Result<(), Self::Err> { Ok(()) }
}

// type NoGpio = NotPresent<gpio::MissingGpio>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(ambassador::Delegate)]
#[delegate(Snapshot)]
pub struct Present<T>(pub T);
impl<T> OptionalPeripheral for Present<T> {
    type Inner = T;
    type Present = True;
    fn get(&self) -> Option<&T> { Some(&self.0) }
    fn get_mut(&mut self) -> Option<&mut T> { Some(&mut self.0) }
}

// impl<G, A, P, T, C, I, O> PeripheralSet<G, A, P, T, C, I, O>
// where
//     G: Gpio,
//     A: Adc,
//     P: Pwm,
//     T: Timers,
//     C: Clock,
//     I: Input,
//     O: Output,
// {
//     pub fn new(gpio: G, adc: A, pwm: P, timers: T, clock: C, input: I, output: O) -> Self {
//         Self {
//             gpio,
//             adc,
//             pwm,
//             timers,
//             clock,
//             input,
//             output,
//             gpio_bank_b: Default::default(),
//             gpio_bank_c: Default::default(),
//         }
//     }
// }

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

// impl<G, A, P, T, C, I, O, GB, GC> Peripherals for PeripheralSet<G, A, P, T, C, I, O, GB, GC>
// where
//     G: Gpio,
//     A: Adc,
//     P: Pwm,
//     T: Timers,
//     C: Clock,
//     I: Input,
//     O: Output,

//     GB: OptionalPeripheral,
//     OptTy<GB>: Gpio,
//     GC: OptionalPeripheral,
//     OptTy<GC>: Gpio,
// {
//     type Gpio = G;
//     type Adc = A;
//     type Pwm = P;
//     type Timers = T;
//     type Clock = C;
//     type Input = I;
//     type Output = O;

//     type GpioB = OptTy<GB>;
//     type GpioC = OptTy<GC>;

//     fn get_gpio(&self) -> &G { &self.gpio }
//     fn get_adc(&self) -> &A { &self.adc }
//     fn get_pwm(&self) -> &P { &self.pwm }
//     fn get_timers(&self) -> &T { &self.timers }
//     fn get_clock(&self) -> &C { &self.clock }
//     fn get_input(&self) -> &I { &self.input }
//     fn get_output(&self) -> &O { &self.output }

//     fn get_gpio_mut(&mut self) -> &mut Self::Gpio { &mut self.gpio }
//     fn get_adc_mut(&mut self) -> &mut Self::Adc { &mut self.adc }
//     fn get_pwm_mut(&mut self) -> &mut Self::Pwm { &mut self.pwm }
//     fn get_timers_mut(&mut self) -> &mut Self::Timers { &mut self.timers }
//     fn get_clock_mut(&mut self) -> &mut Self::Clock { &mut self.clock }
//     fn get_input_mut(&mut self) -> &mut Self::Input { &mut self.input }
//     fn get_output_mut(&mut self) -> &mut Self::Output { &mut self.output }

//     fn get_gpio_bank_b(&self) -> Option<&Self::GpioB> {
//         // To _enforce_ that it's fixed and not variable at runtime.
//         <OptPresent<GB> as Bool>::B.then(|| self.gpio_bank_b.get().unwrap())
//     }

//     fn get_gpio_bank_b_mut(&mut self) -> Option<&mut Self::GpioB> {
//         <OptPresent<GB> as Bool>::B.then(|| self.gpio_bank_b.get_mut().unwrap())
//     }

//     fn get_gpio_bank_c(&self) -> Option<&Self::GpioC> {
//         <OptPresent<GC> as Bool>::B.then(|| self.gpio_bank_c.get().unwrap())
//     }

//     fn get_gpio_bank_c_mut(&mut self) -> Option<&mut Self::GpioC> {
//         <OptPresent<GC> as Bool>::B.then(|| self.gpio_bank_c.get_mut().unwrap())
//     }
// }

use crate::control::{Snapshot, SnapshotError};

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
{
    type Snap = (
        <G as Snapshot>::Snap,
        <A as Snapshot>::Snap,
        <P as Snapshot>::Snap,
        <T as Snapshot>::Snap,
        <C as Snapshot>::Snap,
        <I as Snapshot>::Snap,
        <O as Snapshot>::Snap,
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
