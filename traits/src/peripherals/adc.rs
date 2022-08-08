//! [`Adc` trait](Adc) and associated types.

use lc3_isa::{Word, Bits};
use lc3_macros::DisplayUsingDebug;

use core::convert::TryFrom;
use core::ops::{Deref, Index, IndexMut};

use serde::{Deserialize, Serialize};
// TODO: Add Errors

#[rustfmt::skip]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[derive(DisplayUsingDebug)]
pub enum AdcPin { A0, A1, A2, A3, A4, A5 }

impl AdcPin {
    pub const NUM_PINS: usize = 6;
}

pub const ADC_PINS: AdcPinArr<AdcPin> = {
    use AdcPin::*;
    AdcPinArr([A0, A1, A2, A3, A4, A5])
}; // TODO: once we get the derive macro, get rid of this.

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(DisplayUsingDebug)]
pub enum AdcState {
    Enabled,
    Disabled,
}

impl From<AdcPin> for usize {
    fn from(pin: AdcPin) -> usize {
        use AdcPin::*;
        match pin {
            A0 => 0,
            A1 => 1,
            A2 => 2,
            A3 => 3,
            A4 => 4,
            A5 => 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AdcPinArr<T>(pub [T; AdcPin::NUM_PINS]);

// Once const fn is more stable:
// impl<T: Copy> AdcPinArr<T> {
//     const fn new(val: T) -> Self {
//         Self([val; AdcPin::NUM_PINS])
//     }
// }

impl<T> Deref for AdcPinArr<T> {
    type Target = [T; AdcPin::NUM_PINS];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Index<AdcPin> for AdcPinArr<T> {
    type Output = T;

    fn index(&self, pin: AdcPin) -> &Self::Output {
        &self.0[usize::from(pin)]
    }
}

impl<T> IndexMut<AdcPin> for AdcPinArr<T> {
    fn index_mut(&mut self, pin: AdcPin) -> &mut Self::Output {
        &mut self.0[usize::from(pin)]
    }
}


// represents a 12-bit reading, currently
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[derive(DisplayUsingDebug)]
pub struct AdcReading(Word);

impl From<AdcReading> for Word {
    fn from(r: AdcReading) -> Self {
        r.0
    }
}

impl AdcReading {
    pub const WIDTH: u8 = 12;

    pub const fn new_raw(reading: Word) -> Self {
        let val = reading & 0x0FFFF;
        Self(val)
    }

    #[inline(always)]
    pub fn new<const WIDTH: u8, R: Into<u64>>(reading: R) -> Self {
        let val = reading.into();
        let val = if WIDTH >= Self::WIDTH {
            val >> (WIDTH - Self::WIDTH)
        } else {
            val << (Self::WIDTH - WIDTH)
        };

        let val = val.u16(0..((Self::WIDTH - 1) as u32));

        Self(val)
    }

    pub fn val(self) -> Word { self.0 }
}

/// Adc access for the interpreter.
#[ambassador::delegatable_trait]
pub trait Adc {
    fn set_state(&mut self, pin: AdcPin, state: AdcState) -> Result<(), AdcMiscError>;
    fn get_state(&self, pin: AdcPin) -> AdcState;
    #[inline]
    fn get_states(&self) -> AdcPinArr<AdcState> {
        let mut states = AdcPinArr([AdcState::Disabled; AdcPin::NUM_PINS]);

        ADC_PINS
            .iter()
            .for_each(|a| states[*a] = self.get_state(*a));

        states
    }

    // TODO: remove the pin from these error types, like with GPIO!
    fn read(&self, pin: AdcPin) -> Result<AdcReading, AdcReadError>;
    #[inline]
    fn read_all(&self) -> AdcPinArr<Result<AdcReading, AdcReadError>> {
        // TODO: Error conversion impl (see Gpio)
        let mut readings = AdcPinArr([Err(AdcReadError((AdcPin::A0, AdcState::Disabled))); AdcPin::NUM_PINS]); // TODO: that we need a default value here is weird and bad...

        ADC_PINS
            .iter()
            .for_each(|a| readings[*a] = self.read(*a));

        readings
    }

}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AdcMiscError;

pub type AdcStateMismatch = (AdcPin, AdcState);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AdcReadError(pub AdcStateMismatch);

pub type AdcStateMismatches = AdcPinArr<Option<AdcStateMismatch>>;
impl Copy for AdcStateMismatches { }

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AdcReadErrors(pub AdcStateMismatches);

impl TryFrom<AdcPinArr<Result<u8, AdcReadError>>> for AdcReadErrors {
    type Error = ();

    fn try_from(read_errors: AdcPinArr<Result<u8, AdcReadError>>) -> Result<Self, Self::Error> {
        let mut errors: AdcStateMismatches = AdcPinArr([None; AdcPin::NUM_PINS]);

        read_errors
            .iter()
            .enumerate()
            .filter_map(|(idx, res)| {
                res.map_err(|adc_read_error| (idx, adc_read_error)).err()
            })
            .for_each(|(idx, adc_read_error)| {
                errors.0[idx] = Some(adc_read_error.0);
            });

        Ok(AdcReadErrors(errors))
    }
}
