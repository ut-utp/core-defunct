use core::cell::RefCell;
use core::marker::PhantomData;
use embedded_hal::prelude::_embedded_hal_PwmPin;
use embedded_hal::{PwmPin as hal_pwm_pin};
use lc3_traits::peripherals::pwm::{Pwm as lc3_pwm, *};

// Embedded hal; offers the Pwm and PwmPin trait, but it looks like
// PwmPin and Pwm hal traits don't really have anything to do with each other and are just both
// offered to allow users chose what they want. It's unlike the ADC case where the Oneshot trait did use
// the ADC Channel trait for individual pin references.
// In Pwm case though, there is an associated type for the Pwm trait called channel but using this would need
// custom implementation on that channel associated type to index into and access our pin arrays and this is a needless inconvenience
// Hence, we'll use the PwmPin trait here which has it's own self contained pwm operation functions and create an array of pins.
// PwmPin is also the more mature trait as the hal Pwm trait is marked unproven.
// Only drawback is the TM4C currently doesn't have an implementation for PwmPin (it has one for Pwm) but that is ok. TODO: Make PwmPin impl for TM4C

pub struct GenericPwm<T, U>
where
    T: hal_pwm_pin + _embedded_hal_PwmPin<Duty = U>,
    U: From<u16> + Into<u16>, //adding this trait bound on duty to allow converting to integer form for our platform use. Similar approach as ADC
{
    hal_pins: RefCell<PwmPinArr<T>>,
    pin_states: PwmPinArr<PwmState>,
    phantom: PhantomData<U>,
}

impl<T, U> Default for GenericPwm<T, U>
where
    T: hal_pwm_pin<Duty = U>,
    U: From<u16> + Into<u16>,
{
    fn default() -> Self {
        unimplemented!()
    }
}

impl<T, U> GenericPwm<T, U>
where
    T: hal_pwm_pin<Duty = U>,
    U: From<u16> + Into<u16>,
{
    pub fn new(pwm_device_pins: PwmPinArr<T>) -> Self {
        Self {
            hal_pins: RefCell::new(pwm_device_pins),
            pin_states: PwmPinArr([PwmState::Disabled; PwmPin::NUM_PINS]),
            phantom: PhantomData,
        }
    }
}

impl<T, U> lc3_pwm for GenericPwm<T, U>
where
    T: hal_pwm_pin + _embedded_hal_PwmPin<Duty = U>,
    U: From<u16> + Into<u16>,
{
    fn set_state(&mut self, pin: PwmPin, state: PwmState) {
        let mut hal_pins = self.hal_pins.borrow_mut();
        match state {
            PwmState::Enabled(duty) => {
                hal_pins[pin].enable();
                hal_pins[pin]
                    .set_duty(U::from(core::num::NonZeroU8::get(duty) as u16));
            }
            PwmState::Disabled => {
                hal_pins[pin].disable();
            }
        }
        self.pin_states[pin] = state;
    }
    fn get_state(&self, pin: PwmPin) -> PwmState {
        self.pin_states[pin]
    }
    #[inline]
    fn get_states(&self) -> PwmPinArr<PwmState> {
        let mut states = PwmPinArr([PwmState::Disabled; PwmPin::NUM_PINS]);

        PWM_PINS
            .iter()
            .for_each(|p| states[*p] = self.get_state(*p));

        states
    }

    //Should this function be infallible since there is no error return?
    //For ADC, couldn't read a disabled pin for instance, but since there is no
    //error checking here, just enable and set a pin of it is disabled.
    //And just set duty to 1 if 0 to avoid crashing on unwrap
    fn set_duty_cycle(&mut self, pin: PwmPin, duty_cycle: PwmDutyCycle) {
        let mut duty_cycle_temp = duty_cycle;
        if duty_cycle_temp == 0 {
            duty_cycle_temp = 1;
        }
        self.set_state(
            pin,
            PwmState::Enabled(
                core::num::NonZeroU8::new(duty_cycle_temp).unwrap(),
            ),
        );
    }
    fn get_duty_cycle(&self, pin: PwmPin) -> PwmDutyCycle {
        let state = self.get_state(pin);
        let mut duty_return = core::num::NonZeroU8::new(1).unwrap();
        match state {
            PwmState::Enabled(duty) => {
                duty_return = duty;
            }
            PwmState::Disabled => {}
        }
        core::num::NonZeroU8::get(duty_return)
    }
    #[inline]
    fn get_duty_cycles(&self) -> PwmPinArr<PwmDutyCycle> {
        let mut duty_cycles = PwmPinArr([0u8; PwmPin::NUM_PINS]);

        PWM_PINS
            .iter()
            .for_each(|p| duty_cycles[*p] = self.get_duty_cycle(*p));

        duty_cycles
    }
}
