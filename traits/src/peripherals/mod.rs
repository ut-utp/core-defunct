//! Peripherals! The [`Peripherals` supertrait](peripherals::Peripherals) and the rest of the
//! peripheral and device traits.

pub mod adc;
pub mod clock;
pub mod gpio;
pub mod pwm;
pub mod timers;
pub mod input;
pub mod output;

use core::{marker::PhantomData, convert::Infallible, ops::{Deref, DerefMut}};

pub use gpio::Gpio;
pub use adc::Adc;
pub use pwm::Pwm;
pub use timers::Timers;
pub use clock::Clock;
pub use input::Input;
pub use output::Output;

pub mod stubs;

use super::control::snapshot::SnapshotUsingClone;

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

// TODO: hide the ambassador items in the docs!

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
                paste::paste! {
                    $(#[$opt_peri_attr])*
                    #[doc = "\n\n"]
                    #[doc = "Use [`" $opt_peri_def "`] if you don't want to provide an actual"]
                    #[doc = "[`" $opt_peri_trait "`] implementation!\n"]
                    type $opt_peri_ty_param_name: ?Sized + $opt_peri_trait /* = $opt_peri_def */;
                }
            )*

            // Required peripheral getters:
            $(paste::paste! {
                peripherals!(@getter_pair_def: ($req_peri_name) {
                    #[doc = "for the [`" $req_peri_trait "`] peripheral."]
                } -> &Self::$req_peri_trait, &mut Self::$req_peri_trait = with self rest(;));
            })*

            // Optional peripheral getters:
            $(paste::paste! {
                peripherals!(@getter_pair_def: ($opt_peri_name) {
                    #[doc = " for [`" $nom "::" $opt_peri_ty_param_name "`]"]
                    #[doc = " (the **optional** [`" $opt_peri_trait "`] peripheral)."]
                    #[doc = "\n\n"]
                    #[doc = "Defaults to returning [`None`]; remember to implement this if you \n"]
                    #[doc = "**don't** set [`" $nom "::" $opt_peri_ty_param_name "`] to"]
                    #[doc = "[`" $opt_peri_def "`]."]
                } -> Option<&Self::$opt_peri_ty_param_name>, Option<&mut Self::$opt_peri_ty_param_name> =
                    with self rest({ None })
                );
            })*
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

        /* Define the Peripheral Set: */
        paste::paste! {
            #[derive(ambassador::Delegate)]
            $(
                // Leaning on `paste` to eagerly stringify here (i.e. the `""` below).
                #[delegate($req_peri_trait, target = "" $req_peri_name)]
            )*
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
                    #[doc = "An instance of a(n) [`" $req_peri_trait "`] implementation (**required**)."]
                    #[doc = "\n\n"]
                    #[doc = "Corresponds to [`" $nom "::" $req_peri_trait "`]."]
                    pub $req_peri_name: $req_ty_short,
                )*

                $(
                    #[doc = "An instance of a(n) [`" $opt_peri_trait "`] implementation (**optional**)."]
                    #[doc = "\n\n"]
                    #[doc = "Defaults to [`" $opt_peri_def "`]."]
                    #[doc = "\n\n"]
                    #[doc = "Corresponds to [`" $nom "::" $opt_peri_ty_param_name "`]."]
                    pub $opt_peri_name: $opt_ty_short,
                )*
            }
        }

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
                paste::paste! {
                    #[doc = "Whether or not this [`" $set_ty "`] type has an implementation for"]
                    #[doc = " [`" $nom "::" $opt_peri_ty_param_name "`]."]
                    pub const [< HAS_ $opt_peri_name:snake:upper >]: bool = {
                        <$opt_ty_short as OptionalPeripheral>::Present::B
                    };
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

        /* Snapshot for the Peripheral Set: */
        impl<
            $($req_ty_short,)*
            $($opt_ty_short,)*
        > Snapshot for $set_ty<
            $($req_ty_short,)*
            $($opt_ty_short,)*
        >
        where
            $($req_ty_short: Snapshot + $req_peri_trait,)*
            $(
                $opt_ty_short: Snapshot + OptionalPeripheral,
                OptTy<$opt_ty_short>: $opt_peri_trait,
            )*
        {
            type Snap = (
                $(<$req_ty_short as Snapshot>::Snap,)*
                $(<$opt_ty_short as Snapshot>::Snap,)*
            );

            // TODO: report which thing failed? make it part of the SnapshotError type?
            type Err = SnapshotError;

            fn record(&self) -> Result<Self::Snap, Self::Err> {
                Ok((
                    $( self.$req_peri_name.record().map_err(Into::into)?, )*
                    $( self.$opt_peri_name.record().map_err(Into::into)?, )*
                ))
            }

            fn restore(&mut self, (
                $($req_peri_name,)*
                $($opt_peri_name,)*
            ): Self::Snap) -> Result<(), Self::Err> {
                $( self.$req_peri_name.restore($req_peri_name).map_err(Into::into)?; )*
                $( self.$opt_peri_name.restore($opt_peri_name).map_err(Into::into)?; )*

                Ok(())
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

        /* impl Peripherals and delegate the traits to the wrapper: */
        paste::paste! {
            #[ambassador::delegate_to_methods]
            $(
                // Once again: leaning on `paste` to eagerly stringify here.
                #[delegate(
                    $req_peri_trait,
                    target_ref = "" [< get_ $req_peri_name >],
                    target_mut = "" [< get_ $req_peri_name _mut >],
                )]
            )*
            impl<P: ?Sized + $nom> $nom for $wrapper_ty<P> {
                $(type $req_peri_trait = P::$req_peri_trait;)*
                $(type $opt_peri_ty_param_name = P::$opt_peri_ty_param_name;)*

                $(
                    #[inline(always)] fn [< get_ $req_peri_name >](&self) -> &P::$req_peri_trait
                        { self.0.[< get_ $req_peri_name >]() }
                    #[inline(always)] fn [< get_ $req_peri_name _mut >](&mut self) -> &mut P::$req_peri_trait
                        { self.0.[< get_ $req_peri_name _mut >]() }
                )*

                $(
                    #[inline(always)] fn [< get_ $opt_peri_name >](&self) -> Option<&P::$opt_peri_ty_param_name>
                        { self.0.[< get_ $opt_peri_name >]() }
                    #[inline(always)] fn [< get_ $opt_peri_name _mut >](&mut self) -> Option<&mut P::$opt_peri_ty_param_name>
                        { self.0.[< get_ $opt_peri_name _mut >]() }
                )*
            }
        }

        /* impl traits on SnapshotUsingClone */
        #[ambassador::delegate_remote]
        $( #[delegate($req_peri_trait)] )*
        pub struct SnapshotUsingClone<T: Clone>(T);

        /* impl traits on the `GetRwLock`/`GetMutex` things */
        $(
            peripherals!(make_locked_delegated_impls: $req_peri_name $req_peri_trait ($req_ty_short));
        )*

        // we can automatically do this for the required peripheral traits but
        // not for the optional ones (since they may be duplicates). so: we add
        // static assertions that try to ensure that this has been done for the
        // optional peripheral traits!
        $(
            $(
                sa::assert_impl_all!(
                    core::cell::RefCell<$opt_peri_stub>: $opt_peri_trait
                );
            )? // we can only do this when we've been told the stub type
        )*
    };

    // Note: `ambassador` doesn't support #cfg attrs so we use `using_std_eager` here
    // in conjunction with an _outer_ `using_std` to get us back the `doc_cfg`s
    (make_locked_delegated_impls: $name:ident $trait_name:ident ($s:ident)) => {
        paste::paste! {
            using_std! { mod [< $name _delegated_impls >] {
                use super::*;

                using_std_eager! {
                    use std::{
                        boxed::Box,
                        rc::Rc,
                        sync::{Arc, Mutex, RwLock},
                    };
                    use super::{GetRwLock, GetMutex};

                    peripherals!(@make_locked_delegated_impls
                        ty_param = $s
                        trait = $trait_name
                        list = (
                            RwLock<$s>, Arc<RwLock<$s>>,  Rc<RwLock<$s>>,
                            Mutex<$s>,  Arc<Mutex<$s>>,   Rc<Mutex<$s>>,
                                        Arc<RefCell<$s>>, Rc<RefCell<$s>>,
                            Box<$s>,
                        )
                        funcs = _get, _get_mut
                    );
                }
            }}

            peripherals!(@make_locked_delegated_impls
                ty_param = $s trait = $trait_name
                list = (RefCell<$s>) funcs = _get, _get_mut
            );

            peripherals!(@make_locked_delegated_impls
                ty_param = $s trait = $trait_name
                list = (&mut $s) funcs = _get_from_ref, _get_mut_from_ref
            );
        }
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
            #[doc = "Sets [`" $set_ty "`]'s instance for the optional peripheral"]
            #[doc = " [`" $nom "::" $opt_peri_ty_param_name_first "`] to an instance"]
            #[doc = " of an actual implementation."]
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

    (@make_locked_delegated_impls
        ty_param = $ty_param:ident
        trait = $trait:ident
        list = ($($ty:ty),* $(,)?)
        funcs = $shared:ident, $uniq:ident
    ) => {
        $(paste::paste! {
            #[ambassador::delegate_to_remote_methods]
            #[delegate($trait, target_ref = "" $shared, target_mut = "" $uniq)]
            impl<$ty_param> $ty {
                fn $shared(&self) -> &$ty_param;
                fn $uniq(&mut self) -> &mut $ty_param;
            }
        })*
    }
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

// TODO: fix in upstream
use crate::*;
use self::{gpio::*, adc::*, pwm::*, timers::*, input::*, output::*};
use lc3_isa::Word;

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


pub trait Bool: sealed::Sealed {
    const B: bool;
}
pub struct True; impl Bool for True { const B: bool = true; }
pub struct False; impl Bool for False { const B: bool = false; }

#[doc(hidden)]
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

use crate::control::{Snapshot, SnapshotError};


use core::cell::{RefCell, Ref, RefMut};
trait GetRefCell<I> {
    fn _get(&self) -> Ref<'_, I>;
    fn _get_mut(&self) -> RefMut<'_, I>;
}

impl<I> GetRefCell<I> for RefCell<I> {
    #[inline(always)]
    fn _get(&self) -> Ref<'_, I> { self.borrow() }
    #[inline(always)]
    fn _get_mut(&self) -> RefMut<'_, I> { self.borrow_mut() }
}

// Needs a different trait to avoid name collisions
trait GetFromMutRef<I> {
    fn _get_from_ref(&self) -> &I;
    fn _get_mut_from_ref(&mut self) -> &mut I;
}

impl<I> GetFromMutRef<I> for &mut I {
    #[inline(always)]
    fn _get_from_ref(&self) -> &I { self }
    #[inline(always)]
    fn _get_mut_from_ref(&mut self) -> &mut I { self }
}

using_std! {
    use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Mutex, MutexGuard};
    use std::rc::Rc;

    // Need Arc/Rc impls because Arc/Rc normally don't let us provide these impls;
    // only when it wraps a type that has interior mutability we can.

    // we can't abstract over these types because we don't have GATs (yet)

    // we tell `ambassador::delegate` two lies here:
    //  1) &mut self instead of &self (we have interior mutability)
    //  2) the return type (we actually deref)
    //     - by lying about the return type we don't even need to use `automatic_where_clause`

    trait GetRwLock<I> {
        fn _get(&self) -> RwLockReadGuard<'_, I>;
        fn _get_mut(&self) -> RwLockWriteGuard<'_, I>;
    }

    impl<I> GetRwLock<I> for RwLock<I> {
        #[inline(always)]
        fn _get(&self) -> RwLockReadGuard<'_, I> { RwLock::read(self).unwrap() }
        #[inline(always)]
        fn _get_mut(&self) -> RwLockWriteGuard<'_, I> { RwLock::write(self).unwrap() }
    }

    impl<I, I2: GetRwLock<I>> GetRwLock<I> for Arc<I2> {
        #[inline(always)]
        fn _get(&self) -> RwLockReadGuard<'_, I> { <I2 as GetRwLock<I>>::_get(self) }
        #[inline(always)]
        fn _get_mut(&self) -> RwLockWriteGuard<'_, I> { <I2 as GetRwLock<I>>::_get_mut(self) }
    }

    impl<I, I2: GetRwLock<I>> GetRwLock<I> for Rc<I2> {
        #[inline(always)]
        fn _get(&self) -> RwLockReadGuard<'_, I> { <I2 as GetRwLock<I>>::_get(self) }
        #[inline(always)]
        fn _get_mut(&self) -> RwLockWriteGuard<'_, I> { <I2 as GetRwLock<I>>::_get_mut(self) }
    }

    // impl<I, I2: GetRwLock<I>> GetRwLock<I> for Box<I2> {
    //     fn get(&self) -> RwLockReadGuard<'_, I> { <I2 as GetRwLock<I>>::get(self) }
    //     fn get_mut(&mut self) -> RwLockWriteGuard<'_, I> { <I2 as GetRwLock<I>>::get_mut(self) }
    // }

    trait GetMutex<I> {
        fn _get(&self) -> MutexGuard<'_, I>;
        fn _get_mut(&self) -> MutexGuard<'_, I>;
    }

    impl<I> GetMutex<I> for Mutex<I> {
        #[inline(always)]
        fn _get(&self) -> MutexGuard<'_, I> { Mutex::lock(self).unwrap() }
        #[inline(always)]
        fn _get_mut(&self) -> MutexGuard<'_, I> { Mutex::lock(self).unwrap() }
    }

    impl<I, I2: GetMutex<I>> GetMutex<I> for Arc<I2> {
        #[inline(always)]
        fn _get(&self) -> MutexGuard<'_, I> { <I2 as GetMutex<I>>::_get(self) }
        #[inline(always)]
        fn _get_mut(&self) -> MutexGuard<'_, I> { <I2 as GetMutex<I>>::_get_mut(self) }
    }

    impl<I, I2: GetMutex<I>> GetMutex<I> for Rc<I2> {
        #[inline(always)]
        fn _get(&self) -> MutexGuard<'_, I> { <I2 as GetMutex<I>>::_get(self) }
        #[inline(always)]
        fn _get_mut(&self) -> MutexGuard<'_, I> { <I2 as GetMutex<I>>::_get_mut(self) }
    }

    // impl<I, I2: GetMutex<I>> GetMutex<I> for Box<I2> {
    //     fn get(&self) -> MutexGuard<'_, I> { <I2 as GetMutex<I>>::get(self) }
    //     fn get_mut(&mut self) -> MutexGuard<'_, I> { <I2 as GetMutex<I>>::get_mut(self) }
    // }

    impl<I, I2: GetRefCell<I>> GetRefCell<I> for Arc<I2> {
        #[inline(always)]
        fn _get(&self) -> Ref<'_, I> { <I2 as GetRefCell<I>>::_get(self) }
        #[inline(always)]
        fn _get_mut(&self) -> RefMut<'_, I> { <I2 as GetRefCell<I>>::_get_mut(self) }
    }

    impl<I, I2: GetRefCell<I>> GetRefCell<I> for Rc<I2> {
        #[inline(always)]
        fn _get(&self) -> Ref<'_, I> { <I2 as GetRefCell<I>>::_get(self) }
        #[inline(always)]
        fn _get_mut(&self) -> RefMut<'_, I> { <I2 as GetRefCell<I>>::_get_mut(self) }
    }


    // For Symmetry (makes life easier for the peripherals macro):
    trait GetInner<I> {
        fn _get(&self) -> &I;
        fn _get_mut(&mut self) -> &mut I;
    }

    impl<I> GetInner<I> for Box<I> {
        #[inline(always)]
        fn _get(&self) -> &I { self.deref() }

        #[inline(always)]
        fn _get_mut(&mut self) -> &mut I { self.deref_mut() }
    }
}
