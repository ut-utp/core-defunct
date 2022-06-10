//! [`Gpio` trait](Gpio) and friends.

use lc3_macros::DisplayUsingDebug;

use core::convert::TryFrom;
use core::ops::{Deref, Index, IndexMut};

use serde::{Deserialize, Serialize};

use super::OptionalPeripherals;

#[rustfmt::skip]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[derive(DisplayUsingDebug)]
pub enum GpioBank { A = 0, B = 1, C = 2 }

impl TryFrom<GpioBank> for super::OptionalPeripherals {
    type Error = ();

    fn try_from(value: GpioBank) -> Result<Self, Self::Error> {
        match value {
            GpioBank::A => Err(()),
            GpioBank::B => Ok(OptionalPeripherals::GpioBankB),
            GpioBank::C => Ok(OptionalPeripherals::GpioBankC),
        }
    }
}

impl GpioBank {
    pub const NUM_BANKS: usize = 3; // TODO: derive macro
}
// sa::const_assert_eq!(core::mem::variant_count::<GpioBank>(), GpioPin::NUM_BANKS);


// Switched to using enums to identify peripheral pin numbers; this way
// referring to invalid/non-existent pin numbers isn't an error that peripheral
// trait implementations have to deal with.
//
// This seems to make more sense; consider that the peripherals are exposed
// through a memory-mapped interface an invalid pin number isn't really an error
// that can happen (you either hit a memory address that corresponds to a
// peripheral or you hit an invalid memory address).
//
// This is currently a little wonky, but it'll be better once we write the macro
// described in `control.rs`.

#[rustfmt::skip]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[derive(DisplayUsingDebug)]
pub enum GpioPin { G0, G1, G2, G3, G4, G5, G6, G7 }

impl GpioPin {
    pub const NUM_PINS: usize = 8; // G0 - G7; TODO: derive macro (also get it to impl Display)
}
// sa::const_assert_eq!(core::mem::variant_count::<GpioPin>(), GpioPin::NUM_PINS);

pub const GPIO_PINS: GpioPinArr<GpioPin> = {
    use GpioPin::*;
    GpioPinArr([G0, G1, G2, G3, G4, G5, G6, G7])
}; // TODO: once we get the derive macro, get rid of this.

// TODO: macro!!
impl From<GpioPin> for usize {
    fn from(pin: GpioPin) -> usize {
        use GpioPin::*;

        match pin {
            G0 => 0,
            G1 => 1,
            G2 => 2,
            G3 => 3,
            G4 => 4,
            G5 => 5,
            G6 => 6,
            G7 => 7,
        }
    }
}

// TODO: TryFrom impl

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(DisplayUsingDebug)]
pub enum GpioState {
    Input,
    Output,
    Interrupt, // TBD: Can you call read on a pin configured for interrupts? (TODO)
    // Tentatively, yes.
    //
    // 00 -> Disabled
    // 01 -> Output
    // 10 -> Input
    // 11 -> Interrupt (Rising Edge)
    Disabled,
} // TODO: impl Into<Word>

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GpioPinArr<T>(pub [T; GpioPin::NUM_PINS]);

// For when we have const functions:
// impl<T: Copy> GpioPinArr<T> {
//     const fn new(val: T) -> Self {
//         Self([val; GpioPin::NUM_PINS])
//     }
// }

// Once const fn is more stable:
// impl<T: Copy> GpioPinArr<T> {
//     const fn new(val: T) -> Self {
//         Self([val; GpioPin::NUM_PINS])
//     }
// }

impl<T> Deref for GpioPinArr<T> {
    type Target = [T; GpioPin::NUM_PINS];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Index<GpioPin> for GpioPinArr<T> {
    type Output = T;

    fn index(&self, pin: GpioPin) -> &Self::Output {
        &self.0[usize::from(pin)]
    }
}

impl<T> IndexMut<GpioPin> for GpioPinArr<T> {
    fn index_mut(&mut self, pin: GpioPin) -> &mut Self::Output {
        &mut self.0[usize::from(pin)]
    }
}

// pub type GpioPinArr<T> = [T; GpioPin::NUM_PINS];

//////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GpioMiscError;

// Can't implement `From` because we want `GpioMiscError` to implement `Debug`.
//
// This is okay though; user code doesn't really generate these errors.
impl GpioMiscError {
    pub fn from_source<D: core::fmt::Debug>(_source: D) -> Self {
        // TODO: log!(T)..
        // on the device this can maybe send to the console, idk

        // TODO: on the tm4c, register an env logger that sends stuff to the
        // console!
        GpioMiscError
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GpioReadError {
    IsDisabled,
    IsInOutputMode,
    Other(GpioMiscError),
}

impl From<GpioMiscError> for GpioReadError {
    fn from(misc: GpioMiscError) -> Self {
        GpioReadError::Other(misc)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GpioWriteError {
    IsDisabled,
    IsInInputMode,
    IsInInterruptMode,
    Other(GpioMiscError),
}

impl From<GpioMiscError> for GpioWriteError {
    fn from(misc: GpioMiscError) -> Self {
        GpioWriteError::Other(misc)
    }
}

impl<T: Copy> Copy for GpioPinArr<T> { }

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GpioReadErrors(pub GpioPinArr<Option<GpioReadError>>); // TODO: display impl

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GpioWriteErrors(pub GpioPinArr<Option<GpioWriteError>>); // TODO: display impl

// TODO: conditional (on std) Error impls


// TODO: document all the weird cases
//
// Once a pin is set to output but before a write the pin is? 0? unknown? implementation defined?
// Write to the register in input mode? Ignored
// Read from the register in output mode? 0s? or do we cache the last written value?
//
// current approach:
//  - pins are set to 0 on transition to output
//  - writes to the register in input/interrupt mode error
//  - reads from the output mode return the last written value


/// GPIO access trait. todo: rewrite all of this
///
/// Implementations of this trait must provide digital read, digital write, and rising
/// edge trigger interrupt functionality for 8 GPIO pins which we'll call G0 - G7.
///
/// Implementors of this trait are encouraged to provide an implementation of
/// [`Default`](core::default::Default), if possible. Implementors are also free
/// (and encouraged!) to provide inherent methods on their implementation that
/// allow for configuration of the peripheral.
///
/// ### State
/// The interpreter (user of this trait) will set the states of all the pins to
/// [`GpioState::Disabled`] on startup, so implementations can choose any default state
/// they wish.
///
/// Implementations should maintain the state of the GPIO pins and querying this state
/// ([`get_state`](Gpio::get_state)) should be an infallible operation.
///
/// Setting pin state ([`set_state`](Gpio::set_state)) is not infallible as
/// implementations may need to actually change the state of hardware peripherals
/// in order to, for example, register a rising-edge interrupt for a particular pin.
/// Though implementors are encouraged to make this operation infallible if possible, we
/// realize this isn't always possible and in the event that it isn't, we'd rather have
/// implementors pass the error onto the interpreter instead of panicking.
///
/// ### Reads and Writes
/// Reading from pins should fail (with a [`GpioReadError`]) when pins are disabled or
/// in output ([`GpioState::Output`]) mode. *Note:* reading from pins in interrupt
/// ([`GpioState::Interrupt`]) mode is allowed.
///
/// Writing from pins should fail (with a [`GpioWriteError`]) when pins are disabled or
/// in input ([`GpioState::Input`]) or interrupt ([`GpioState::Interrupt`]) mode.
///
/// ### Interrupts
/// TODO: OUT OF DATE. !!
/// Registering interrupts (i.e. calling
/// [`register_interrupt`](Gpio::register_interrupt)) does not automatically put a pin
/// in [`interrupt`](GpioState::Interrupt) mode. Instead, this only updates the handler
/// function for a pin.
///
/// Implementations should store the last handler function provided to
/// [`register_interrupt`](Gpio::register_interrupt) _across pin state changes_. As in,
/// if G0 (GPIO pin 0)'s handler is set to function A (i.e.
/// `register_interrupt(GpioPin::G0, A)`), and then G0's state is changed to
/// [`output`](GpioState::Output) and then to [`disabled`](GpioState::Disabled) and then
/// to [`interrupt`](GpioState::Interrupt), A should be called when G0 goes from low to
/// high.
///
/// Implementors should use a no-op handler (do nothing) for the pins by default. All
/// users of this trait _should_ register handlers on initialization (just as they will
/// explicitly set the state of all pins to [disabled](GpioState::Disabled)), but
/// implementors shouldn't bank on this.
///
/// As is probably obvious, implementors should only call the handler function on a
/// rising edge *if the pin is in [interrupt](GpioState::Interrupt) mode* (not just if
/// a handler function has been provided).
///
/// ### Default Function Implementations
/// The trait provides naïve default implementations of
/// [`get_states`](Gpio::get_states), [`read_all`](Gpio::read_all), and
/// [`write_all`](Gpio::write_all) that just call their single pin variants across all
/// pins; as an implementor you can choose to override these if you wish. If there's an
/// easier way to do a particular operation across all the pins than just calling the
/// single pin variant in a loop, then it's probably worth doing; i.e. if you happen to
/// store [`GpioState`]s for the pins in an array, you could override
/// [`get_states`](Gpio::get_states) to just return your array pretty easily. Otherwise,
/// the default implementations should work fine.
///
/// ### Tests
/// There are [tests for this trait](crate::tests::gpio) in the [tests
/// module](crate::tests) to help ensure that your implementation of this trait follows
/// the rules above. (TODO: this isn't true anymore?)
#[ambassador::delegatable_trait]
pub trait Gpio {
    fn set_state(&mut self, pin: GpioPin, state: GpioState) -> Result<(), GpioMiscError>; // should probably be infallible
    fn get_state(&self, pin: GpioPin) -> GpioState;

    #[inline]
    fn get_states(&self) -> GpioPinArr<GpioState> {
        let mut states = GpioPinArr([GpioState::Disabled; GpioPin::NUM_PINS]); // TODO (again)

        GPIO_PINS
            .iter()
            .for_each(|g| states[*g] = self.get_state(*g));

        states
    }

    // TODO: take &mut self?
    fn read(&self, pin: GpioPin) -> Result<bool, GpioReadError>; // errors on state mismatch (i.e. you tried to read but the pin is configured as an output)
    #[inline]
    fn read_all(&self) -> GpioPinArr<Result<bool, GpioReadError>> {
        // TODO: here's a thing; [Result<bool, GpioReadError>] or Result<[bool], [GpioReadError]>?
        // The interpreter will _probably_ just use a default value upon encountering read errors
        // meaning that we don't want to do the latter which is all or nothing (i.e. if some of the
        // reads worked, give us their values! We'll use them!).

        let mut readings = GpioPinArr([Ok(false); GpioPin::NUM_PINS]); // TODO: it's weird and gross that we have to use a default value here (derive macro save us pls!!)

        GPIO_PINS
            .iter()
            .for_each(|g| readings[*g] = self.read(*g));

        readings
    }

    fn write(&mut self, pin: GpioPin, bit: bool) -> Result<(), GpioWriteError>; // errors on state mismatch
    #[inline]
    fn write_all(&mut self, bits: GpioPinArr<bool>) -> GpioPinArr<Result<(), GpioWriteError>> {
        // TODO: return an array of results or one result?
        // For the actual interpreter, it doesn't make a difference; we have no mechanism by which
        // we even communicate errors to the LC-3 program. But the debugger can communicate this kind
        // of stuff so let's not throw the information away.

        let mut errors = GpioPinArr([Ok(()); GpioPin::NUM_PINS]);

        GPIO_PINS
            .iter()
            .zip(bits.iter())
            .for_each(|(pin, bit)| {
                errors[*pin] = self.write(*pin, *bit);
            });

        errors
    }


    // should we have the interpreter track this state (i.e. when an interrupt is pending)?
    //
    // i think not; it's _conceivable_ that hardware would _want_ to do something
    // weird here (even though the most likely desired behavior is just "interrupt pending
    // until handled")
    //
    // we also don't want to have to deal with duplicated state, etc.
    //
    // we should, however, specify behavior for edge cases like "what happens if there's a pending interrupt
    // that's not handled isn't handled before the user switches the mode to not-Interrupt mode?"
    // (the answer is: `interrupt_occurred` should now return false).
    //
    //
    // I actually think we should maybe update this interface to reflect that this is the behavior..
    fn interrupt_occurred(&self, pin: GpioPin) -> bool;
    fn reset_interrupt_flag(&mut self, pin: GpioPin);

    // TODO: is there a reason why this method exists?
    //
    // I think we initially were reticient to bake in `AtomicBool` as the interrupt
    // signaling mechanism but... I think it's fine to, actually.
    //
    // concern is the cortex-m0 I guess (and other weirder platforms..)
    #[inline]
    fn interrupts_enabled(&self, pin: GpioPin) -> bool {
        matches!(self.get_state(pin), GpioState::Interrupt)
    }
}

// TODO: is `TryFrom` the best way to expose this functionality?
impl TryFrom<GpioPinArr<Result<bool, GpioReadError>>> for GpioReadErrors {
    type Error = ();

    fn try_from(
        read_errors: GpioPinArr<Result<bool, GpioReadError>>,
    ) -> Result<GpioReadErrors, ()> {
        if read_errors.iter().all(|r| r.is_ok()) {
            Err(()) // No error!
        } else {
            let mut errors = GpioPinArr([None; GpioPin::NUM_PINS]);

            read_errors
                .iter()
                .enumerate()
                .filter_map(|(idx, res)| {
                    res.map_err(|gpio_read_error| (idx, gpio_read_error)).err()
                })
                .for_each(|(idx, gpio_read_error)| {
                    errors.0[idx] = Some(gpio_read_error);
                });

            Ok(GpioReadErrors(errors))
        }
    }
}

impl TryFrom<GpioPinArr<Result<(), GpioWriteError>>> for GpioWriteErrors {
    type Error = ();

    fn try_from(
        write_errors: GpioPinArr<Result<(), GpioWriteError>>,
    ) -> Result<GpioWriteErrors, ()> {
        if write_errors.iter().all(|w| w.is_ok()) {
            // None
            Err(())
        } else {
            let mut errors = GpioPinArr([None; GpioPin::NUM_PINS]);

            write_errors
                .iter()
                .enumerate()
                .filter_map(|(idx, res)| {
                    res.map_err(|gpio_write_error| (idx, gpio_write_error))
                        .err()
                })
                .for_each(|(idx, gpio_write_error)| {
                    errors.0[idx] = Some(gpio_write_error);
                });

            // Some(GpioWriteErrors(errors))
            Ok(GpioWriteErrors(errors))
        }
    }
}
