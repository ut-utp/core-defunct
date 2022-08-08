//! [`Pwm` trait](Pwm) and helpers.

use lc3_macros::DisplayUsingDebug;

use core::num::NonZeroU8;
use core::ops::{Deref, Index, IndexMut};

use serde::{Deserialize, Serialize};

#[rustfmt::skip]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[derive(DisplayUsingDebug)]
pub enum PwmPin { P0, P1 }

// TODO: remove once the derive macro happens...
impl PwmPin {
    pub const NUM_PINS: usize = 2; // P0 - P1
}

impl From<PwmPin> for usize {
    fn from(pin: PwmPin) -> usize {
        use PwmPin::*;
        match pin {
            P0 => 0,
            P1 => 1,
        }
    }
}

pub const PWM_PINS: PwmPinArr<PwmPin> = {
    use PwmPin::*;
    PwmPinArr([P0, P1])
}; // TODO: save us, derive macro

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PwmState {
    Enabled(NonZeroU8),
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PwmPinArr<T>(pub [T; PwmPin::NUM_PINS]);

// Once const fn is more stable:
// impl<T: Copy> PwmPinArr<T> {
//     const fn new(val: T) -> Self {
//         Self([val; PwmPin::NUM_PINS])
//     }
// }

impl<T> Deref for PwmPinArr<T> {
    type Target = [T; PwmPin::NUM_PINS];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Index<PwmPin> for PwmPinArr<T> {
    type Output = T;

    fn index(&self, pin: PwmPin) -> &Self::Output {
        &self.0[usize::from(pin)]
    }
}

impl<T> IndexMut<PwmPin> for PwmPinArr<T> {
    fn index_mut(&mut self, pin: PwmPin) -> &mut Self::Output {
        &mut self.0[usize::from(pin)]
    }
}

pub type PwmDutyCycle = u8;

#[ambassador::delegatable_trait]
pub trait Pwm {
    fn set_state(&mut self, pin: PwmPin, state: PwmState);
    fn get_state(&self, pin: PwmPin) -> PwmState;
    #[inline]
    fn get_states(&self) -> PwmPinArr<PwmState> {
        let mut states = PwmPinArr([PwmState::Disabled; PwmPin::NUM_PINS]);

        PWM_PINS
            .iter()
            .for_each(|p| states[*p] = self.get_state(*p));

        states
    }

    fn set_duty_cycle(&mut self, pin: PwmPin, duty_cycle: PwmDutyCycle);
    fn get_duty_cycle(&self, pin: PwmPin) -> PwmDutyCycle;
    #[inline]
    fn get_duty_cycles(&self) -> PwmPinArr<PwmDutyCycle> {
        let mut duty_cycles = PwmPinArr([0u8; PwmPin::NUM_PINS]);

        PWM_PINS
            .iter()
            .for_each(|p| duty_cycles[*p] = self.get_duty_cycle(*p));

        duty_cycles
    }
}
