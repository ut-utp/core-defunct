use core::ops::{Index, IndexMut};
use lc3_traits::peripherals::gpio::{
    Gpio, GpioMiscError, GpioPin, GpioPinArr, GpioReadError, GpioState, GpioWriteError,
    NUM_GPIO_PINS,
};
use std::sync::{Arc, RwLock};

#[derive(Copy, Clone, Debug)]
pub enum State {
    Input(bool),
    Output(bool),
    Interrupt(bool),
    Disabled,
}

impl From<State> for GpioState {
    fn from(state: State) -> GpioState {
        use GpioState::*;

        match state {
            State::Input(_) => Input,
            State::Output(_) => Output,
            State::Interrupt(_) => Interrupt,
            State::Disabled => Disabled,
        }
    }
}

/// A simple reference implementation of the [`Gpio` peripheral
/// trait](crate::peripherals::Gpio).
///
/// Some implementation details:
///   - The value of a pin is set to 0 when the pin is switched to input,
///     output, or interrupt mode.
///   - The value of a pin can be read when the pin is in input, output, or
///     interrupt mode (anything but disabled).
///   - The value of a pin can be _set_ when the pin is in input or interrupt
///     mode.
///   - The state of a pin (input, output, interrupt, or disabled) can be
///     retrieved at any time.
pub struct GpioShim<'a> {
    states: GpioPinArr<State>,
    handlers: GpioPinArr<&'a dyn Fn(GpioPin)>,
}

impl Index<GpioPin> for GpioShim<'_> {
    type Output = State;

    fn index(&self, pin: GpioPin) -> &State {
        &self.states[Into::<usize>::into(pin)]
    }
}

impl IndexMut<GpioPin> for GpioShim<'_> {
    fn index_mut(&mut self, pin: GpioPin) -> &mut State {
        &mut self.states[Into::<usize>::into(pin)]
    }
}

const NO_OP: &dyn Fn(GpioPin) = &|_| {};

impl Default for GpioShim<'_> {
    fn default() -> Self {
        Self {
            states: [State::Disabled; NUM_GPIO_PINS as usize],
            // handlers: [Box::new(&|_| {}); NUM_GPIO_PINS as usize],
            // handlers: [no_op; NUM_GPIO_PINS as usize],
            handlers: [NO_OP; NUM_GPIO_PINS as usize],
        }
    }
}

impl GpioShim<'_> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_shared() -> Arc<RwLock<Self>> {
        Arc::<RwLock<Self>>::default()
    }

    /// Sets a pin if it's in input or interrupt mode.
    ///
    /// Returns `Some(())` on success and `None` on failure.
    pub fn set_pin(&mut self, pin: GpioPin, bit: bool) -> Option<()> {
        use State::*;

        self[pin] = match self[pin] {
            Input(_) => Input(bit),
            Interrupt(prev) => {
                // Rising edge!
                if bit && !prev {
                    self.handlers[Into::<usize>::into(pin)](pin)
                }

                Interrupt(bit)
            }
            Output(_) | Disabled => return None,
        };

        Some(())
    }

    /// Gets the value of a pin.
    ///
    /// Returns `None` when the pin is disabled.
    pub fn get_pin(&self, pin: GpioPin) -> Option<bool> {
        use State::*;

        match self[pin] {
            Input(b) | Output(b) | Interrupt(b) => Some(b),
            Disabled => None,
        }
    }

    /// Gets the state of a pin. Infallible.
    pub fn get_pin_state(&self, pin: GpioPin) -> GpioState {
        self[pin].into()
    }
}

impl<'a> Gpio<'a> for GpioShim<'a> {
    fn set_state(&mut self, pin: GpioPin, state: GpioState) -> Result<(), GpioMiscError> {
        use GpioState::*;
       self[pin] = match state {
            Input => State::Input(false),
            Output => State::Output(false),
            Interrupt => State::Interrupt(false),
            Disabled => State::Disabled,
        };

        Ok(())
    }

    fn get_state(&self, pin: GpioPin) -> GpioState {
        self.get_pin_state(pin)
    }

    fn read(&self, pin: GpioPin) -> Result<bool, GpioReadError> {
        use State::*;

        if let Input(b) | Interrupt(b) = self[pin] {
            Ok(b)
        } else {
            Err(GpioReadError((pin, self[pin].into())))
        }
    }

    fn write(&mut self, pin: GpioPin, bit: bool) -> Result<(), GpioWriteError> {
        use State::*;

        if let Output(_) = self[pin] {
            self[pin] = Output(bit);
            Ok(())
        } else {
            Err(GpioWriteError((pin, self[pin].into())))
        }
    }

    fn register_interrupt(
        &mut self,
        pin: GpioPin,
        handler: &'a (dyn Fn(GpioPin) + Send),
    ) -> Result<(), GpioMiscError> {
        self.handlers[Into::<usize>::into(pin)] = handler;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use lc3_traits::peripherals::gpio::{self, Gpio, GpioPin::*,};


    #[test]
    fn get_state_disabled() {
        let shim = GpioShim::new();
        assert_eq!(shim.get_state(G0), gpio::GpioState::Disabled)
    }

    #[test]
    fn read_input() {
        let mut shim = GpioShim::new();
        let res = shim.set_state(G0, gpio::GpioState::Input);
        assert_eq!(res, Ok(()));
        let val = shim.read(G0);
        assert_eq!(val, Ok(false));
    }

    #[test]
    fn read_interrupt() {
        let mut shim = GpioShim::new();
        let res = shim.set_state(G0, gpio::GpioState::Interrupt);
        assert_eq!(res, Ok(()));
        let val = shim.read(G0);
        assert_eq!(val, Ok(false));
    }
     #[test]
    fn read_disabled() {
        let mut shim = GpioShim::new();
        let val = shim.read(G0);
        assert_eq!(val, Err(GpioReadError((G0, gpio::GpioState::Disabled))));
    }

    //   #[test]
    // fn register_interrupt_test() {
    //     let mut shim = GpioShim::new();
    //     let res = shim.set_state(G0, gpio::GpioState::Interrupt);
    //     assert_eq!(res, Ok(()));
    //     res = shim.register_interrupt(G0, &interrupt_test(G0));
    //     let val = shim.read(G0);
    //     assert_eq!(val, Ok(false));
    // }

    // covers read for output
    #[test]
    fn write_Output() {
        let mut shim = GpioShim::new();
        let res = shim.set_state(G0, gpio::GpioState::Output);
        assert_eq!(res, Ok(()));
        let result = shim.write(G0, true);
        let val = shim.read(G0);
        assert_eq!(val, Err(GpioReadError((G0, gpio::GpioState::Output))));
    }
 // covers read for output
    #[test]
    fn write_Else() {
        let mut shim = GpioShim::new();
        let res = shim.set_state(G0, gpio::GpioState::Input);
        assert_eq!(res, Ok(()));
        let result = shim.write(G0, true);
        assert_eq!(result, Err(GpioWriteError((G0, gpio::GpioState::Input))));
    }



}




// TODO: Either newtype this or make this a blanket impl and move it to `lc3-traits` behind a feature gate (that this crate then requires).
// impl<'a> Gpio<'a> for Arc<RwLock<GpioShim<'a>>> {
//     fn set_state(&mut self, pin: GpioPin, state: GpioState) -> Result<(), GpioMiscError> {
//         RwLock::write(self).unwrap().set_state(pin, state)
//     }

//     fn get_state(&self, pin: GpioPin) -> GpioState {
//         RwLock::read(self).unwrap().get_state(pin)
//     }

//     fn read(&self, pin: GpioPin) -> Result<bool, GpioReadError> {
//         RwLock::read(self).unwrap().read(pin)
//     }

//     fn write(&mut self, pin: GpioPin, bit: bool) -> Result<(), GpioWriteError> {
//         RwLock::write(self).unwrap().write(pin, bit)
//     }

//     fn register_interrupt(
//         &mut self,
//         pin: GpioPin,
//         handler: &'a (dyn Fn(GpioPin) + Send),
//     ) -> Result<(), GpioMiscError> {
//         RwLock::write(self)
//             .unwrap()
//             .register_interrupt(pin, handler)
//     }
// }
