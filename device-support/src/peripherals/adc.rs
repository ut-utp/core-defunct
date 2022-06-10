use lc3_traits::peripherals::adc::{AdcPin, AdcReading, Adc, AdcMiscError, AdcReadError, AdcState, AdcPinArr, ADC_PINS};

extern crate embedded_hal;

use core::cell::RefCell;
use core::marker::PhantomData;
use embedded_hal::adc::{Channel, OneShot};

pub struct GenericAdc<
    const W: u8 /*  = 12 */,
    U,
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    WORD,
    ADC,
> where
    A0: Channel<ADC>,
    A1: Channel<ADC>,
    A2: Channel<ADC>,
    A3: Channel<ADC>,
    A4: Channel<ADC>,
    A5: Channel<ADC>,

    U: OneShot<ADC, WORD, A0>
        + OneShot<ADC, WORD, A1>
        + OneShot<ADC, WORD, A2>
        + OneShot<ADC, WORD, A3>
        + OneShot<ADC, WORD, A4>
        + OneShot<ADC, WORD, A5>,
    WORD: From<u16> + Into<u16>,
{
    adc_unit: RefCell<U>,
    a0: RefCell<A0>,
    a1: RefCell<A1>,
    a2: RefCell<A2>,
    a3: RefCell<A3>,
    a4: RefCell<A4>,
    a5: RefCell<A5>,
    pin_states: AdcPinArr<AdcState>,
    phantom: PhantomData<WORD>,
    phantom2: PhantomData<ADC>,
}

impl<const W: u8, U, A0, A1, A2, A3, A4, A5, WORD, ADC>
    GenericAdc<W, U, A0, A1, A2, A3, A4, A5, WORD, ADC>
where
    A0: Channel<ADC>,
    A1: Channel<ADC>,
    A2: Channel<ADC>,
    A3: Channel<ADC>,
    A4: Channel<ADC>,
    A5: Channel<ADC>,
    U: OneShot<ADC, WORD, A0>
        + OneShot<ADC, WORD, A1>
        + OneShot<ADC, WORD, A2>
        + OneShot<ADC, WORD, A3>
        + OneShot<ADC, WORD, A4>
        + OneShot<ADC, WORD, A5>,
    WORD: From<u16> + Into<u16>,
{
    pub fn new(
        hal_adc: U,
        p0: A0,
        p1: A1,
        p2: A2,
        p3: A3,
        p4: A4,
        p5: A5,
    ) -> Self {
        Self {
            adc_unit: RefCell::new(hal_adc),
            a0: RefCell::new(p0),
            a1: RefCell::new(p1),
            a2: RefCell::new(p2),
            a3: RefCell::new(p3),
            a4: RefCell::new(p4),
            a5: RefCell::new(p5),
            pin_states: AdcPinArr([AdcState::Disabled; AdcPin::NUM_PINS]),
            phantom: PhantomData,
            phantom2: PhantomData,
        }
    }
}

macro_rules! adc_read_pin {
    ($pin:ident, $self:ident, $width:ident) => {{
        let mut pin = $self.$pin.borrow_mut();
        let mut adc = $self.adc_unit.borrow_mut();
        let result = adc.read(&mut *pin);

        match result{
            Ok(value) => Ok(AdcReading::new::<$width, u16>(value.into())),
            _ => {
                // Err(AdcReadError((pin, AdcState::Disabled))) would return this error as this is the default value of $adc_reading
                // TODO: This is not the correct eror type. should be miscallaneous error for HAL read fail? Requires a trait modification
                panic!("uh-oh");
            },
        }
    }}
}

impl<const W: u8, U, A0, A1, A2, A3, A4, A5, WORD, ADC> Adc
    for GenericAdc<W, U, A0, A1, A2, A3, A4, A5, WORD, ADC>
where
    A0: Channel<ADC>,
    A1: Channel<ADC>,
    A2: Channel<ADC>,
    A3: Channel<ADC>,
    A4: Channel<ADC>,
    A5: Channel<ADC>,
    U: OneShot<ADC, WORD, A0>
        + OneShot<ADC, WORD, A1>
        + OneShot<ADC, WORD, A2>
        + OneShot<ADC, WORD, A3>
        + OneShot<ADC, WORD, A4>
        + OneShot<ADC, WORD, A5>,

    WORD: From<u16> + Into<u16>, // This trait bound is the only additional constraint being imposed on the HAL generics
                                 // to make them usable with our traits. This is necessary since the HAL trait definition
                                 // does not impose any bounds on WORD but we need to get an integer from it to use it with our platform
                                 // Hence, this might require one small additional custom board specific implementation to the HAL traits to convert
                                 // the WORD type they use to/from u16 (if the board specific hal impl doesn't already implement it for its WORD).
                                 // u16 was picked as it seems like a reasonable type for adc readings, and the embedded-hal
                                 // official example also uses u16 for adc reading.
{
    // HAL has no notion of state/enabled or disabled pins. So just maintaining the pin state in as a pin array software
    // Therefore, Users will still have to enable pins to use them, but this implementation will just return Ok(()) always
    // since no operation is done on the actual adc peripheral. just infallible software operations
    fn set_state(
        &mut self,
        pin: AdcPin,
        state: AdcState,
    ) -> Result<(), AdcMiscError> {
        //let mut pins = self.pin_.borrow_mut();
        self.pin_states[pin] = state;
        Ok(())
    }
    fn get_state(&self, pin: AdcPin) -> AdcState {
        //let mut pins = self.pins.borrow_mut();
        self.pin_states[pin]
    }

    #[inline]
    fn get_states(&self) -> AdcPinArr<AdcState> {
        let mut states = AdcPinArr([AdcState::Disabled; AdcPin::NUM_PINS]);

        ADC_PINS
            .iter()
            .for_each(|a| states[*a] = self.get_state(*a));

        states
    }

    fn read(&self, pin: AdcPin) -> Result<AdcReading, AdcReadError> {
        use AdcPin::*;
        use AdcState::*;

        match self.get_state(pin) {
            d @ Disabled => Err(AdcReadError((pin, d))),
            Enabled => match pin {
                A0 => adc_read_pin!(a0, self, W),
                A1 => adc_read_pin!(a1, self, W),
                A2 => adc_read_pin!(a2, self, W),
                A3 => adc_read_pin!(a3, self, W),
                A4 => adc_read_pin!(a4, self, W),
                A5 => adc_read_pin!(a5, self, W),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate embedded_hal;
    extern crate embedded_hal_mock;

    use super::*;

    use embedded_hal_mock::adc::Mock;
    use embedded_hal_mock::adc::Transaction;
    use embedded_hal_mock::adc::{MockChan0, MockChan1, MockChan2};

    #[test]
    fn basic_test() {
        let expectations = [
            Transaction::read(0, 0xa0),
            Transaction::read(1, 0xa0),
            Transaction::read(2, 0xa0),
            Transaction::read(0, 0xa0),
            Transaction::read(1, 0xa0),
            Transaction::read(2, 0xa0),
        ];

        let mut generic_adc = GenericAdc::<12, _, _, _, _, _, _, _, _, _>::new(
            Mock::<u16>::new(&expectations),
            MockChan0,
            MockChan1,
            MockChan2,
            MockChan0.clone(),
            MockChan1.clone(),
            MockChan2.clone(),
        );
        generic_adc.set_state(AdcPin::A0, AdcState::Enabled).unwrap();
        assert_eq!(generic_adc.read(AdcPin::A0), Ok(AdcReading::new::<12, _>(0xA0u16)));
    }

    #[test]
    fn disabled_adc() {
        let expectations = [
            Transaction::read(0, 0xa0),
            Transaction::read(1, 0xa0),
            Transaction::read(2, 0xa0),
            Transaction::read(0, 0xa0),
            Transaction::read(1, 0xa0),
            Transaction::read(2, 0xa0),
        ];

        let mut generic_adc = GenericAdc::<12, _, _, _, _, _, _, _, _, _>::new(
            Mock::<u16>::new(&expectations),
            MockChan0,
            MockChan1,
            MockChan2,
            MockChan0.clone(),
            MockChan1.clone(),
            MockChan2.clone(),
        );
        //generic_adc.set_state(AdcPin::A0, AdcState::Enabled);

        assert_eq!(
            generic_adc.read(AdcPin::A0),
            Err(AdcReadError((AdcPin::A0, AdcState::Disabled)))
        );
        assert_eq!(
            generic_adc.read(AdcPin::A1),
            Err(AdcReadError((AdcPin::A1, AdcState::Disabled)))
        );
        assert_eq!(
            generic_adc.read(AdcPin::A2),
            Err(AdcReadError((AdcPin::A2, AdcState::Disabled)))
        );
        assert_eq!(
            generic_adc.read(AdcPin::A3),
            Err(AdcReadError((AdcPin::A3, AdcState::Disabled)))
        );
        assert_eq!(
            generic_adc.read(AdcPin::A4),
            Err(AdcReadError((AdcPin::A4, AdcState::Disabled)))
        );
        assert_eq!(
            generic_adc.read(AdcPin::A5),
            Err(AdcReadError((AdcPin::A5, AdcState::Disabled)))
        );

        generic_adc.set_state(AdcPin::A0, AdcState::Disabled).unwrap();
        generic_adc.set_state(AdcPin::A1, AdcState::Disabled).unwrap();
        generic_adc.set_state(AdcPin::A2, AdcState::Disabled).unwrap();
        generic_adc.set_state(AdcPin::A3, AdcState::Disabled).unwrap();
        generic_adc.set_state(AdcPin::A4, AdcState::Disabled).unwrap();
        generic_adc.set_state(AdcPin::A5, AdcState::Disabled).unwrap();

        let readings = generic_adc.read_all().0;

        //A from<usize> for AdcPin impl would have simplified this
        assert_eq!(
            readings[0],
            Err(AdcReadError((AdcPin::A0, AdcState::Disabled)))
        );
        assert_eq!(
            readings[1],
            Err(AdcReadError((AdcPin::A1, AdcState::Disabled)))
        );
        assert_eq!(
            readings[2],
            Err(AdcReadError((AdcPin::A2, AdcState::Disabled)))
        );
        assert_eq!(
            readings[3],
            Err(AdcReadError((AdcPin::A3, AdcState::Disabled)))
        );
        assert_eq!(
            readings[4],
            Err(AdcReadError((AdcPin::A4, AdcState::Disabled)))
        );
        assert_eq!(
            readings[5],
            Err(AdcReadError((AdcPin::A5, AdcState::Disabled)))
        );
    }

    #[test]
    fn readings_test() {
        let expectations = [
            Transaction::read(0, 0xa0),
            Transaction::read(1, 0xa0b1),
            Transaction::read(2, 0xb0c2),
            Transaction::read(0, 0xc0d3),
            Transaction::read(1, 0xd0e4),
            Transaction::read(2, 0xffff),
        ];

        let mut generic_adc = GenericAdc::<12, _, _, _, _, _, _, _, _, _>::new(
            Mock::<u16>::new(&expectations),
            MockChan0,
            MockChan1,
            MockChan2,
            MockChan0.clone(),
            MockChan1.clone(),
            MockChan2.clone(),
        );
        generic_adc.set_state(AdcPin::A0, AdcState::Enabled).unwrap();
        generic_adc.set_state(AdcPin::A1, AdcState::Enabled).unwrap();
        generic_adc.set_state(AdcPin::A2, AdcState::Enabled).unwrap();
        generic_adc.set_state(AdcPin::A3, AdcState::Enabled).unwrap();
        generic_adc.set_state(AdcPin::A4, AdcState::Enabled).unwrap();
        generic_adc.set_state(AdcPin::A5, AdcState::Enabled).unwrap();

        let readings = generic_adc.read_all().0;

        assert_eq!(readings[0], Ok(AdcReading::new::<12, u16>(0x0a)));
        assert_eq!(readings[1], Ok(AdcReading::new::<12, u16>(0x0b)));
        assert_eq!(readings[2], Ok(AdcReading::new::<12, u16>(0x0c)));
        assert_eq!(readings[3], Ok(AdcReading::new::<12, u16>(0x0d)));
        assert_eq!(readings[4], Ok(AdcReading::new::<12, u16>(0x0e)));
        assert_eq!(readings[5], Ok(AdcReading::new::<12, u16>(0xff)));
    }

    //exercises set_state, get_state and get_states methods
    #[test]
    fn state_test() {
        let expectations = [
            Transaction::read(0, 0xa0),
            Transaction::read(1, 0xa0b1),
            Transaction::read(2, 0xb0c2),
            Transaction::read(0, 0xc0d3),
            Transaction::read(1, 0xd0e4),
            Transaction::read(2, 0xffff),
        ];

        let mut generic_adc = GenericAdc::<12, _, _, _, _, _, _, _, _, _>::new(
            Mock::<u16>::new(&expectations),
            MockChan0,
            MockChan1,
            MockChan2,
            MockChan0.clone(),
            MockChan1.clone(),
            MockChan2.clone(),
        );
        generic_adc.set_state(AdcPin::A0, AdcState::Enabled).unwrap();
        generic_adc.set_state(AdcPin::A1, AdcState::Enabled).unwrap();
        generic_adc.set_state(AdcPin::A2, AdcState::Disabled).unwrap();
        generic_adc.set_state(AdcPin::A3, AdcState::Enabled).unwrap();
        generic_adc.set_state(AdcPin::A4, AdcState::Disabled).unwrap();

        assert_eq!(generic_adc.get_state(AdcPin::A0), AdcState::Enabled);
        assert_eq!(generic_adc.get_state(AdcPin::A1), AdcState::Enabled);
        assert_eq!(generic_adc.get_state(AdcPin::A2), AdcState::Disabled);

        let states = generic_adc.get_states().0;
        assert_eq!(states[0], AdcState::Enabled);
        assert_eq!(states[1], AdcState::Enabled);
        assert_eq!(states[2], AdcState::Disabled);
        assert_eq!(states[3], AdcState::Enabled);
        assert_eq!(states[4], AdcState::Disabled);
        assert_eq!(states[5], AdcState::Disabled);
    }

    #[test]
    fn mixed_test() {
        let expectations = [
            Transaction::read(0, 0x01),
            //Transaction::read(1, 0xa0b1),
            Transaction::read(2, 0xeee4),
            Transaction::read(0, 0xc0d3),
            // Transaction::read(1, 0xd0e4),
            Transaction::read(2, 0xffff),
        ];

        let mut generic_adc = GenericAdc::<12, _, _, _, _, _, _, _, _, _>::new(
            Mock::<u16>::new(&expectations),
            MockChan0,
            MockChan1,
            MockChan2,
            MockChan0.clone(),
            MockChan1.clone(),
            MockChan2.clone(),
        );
        generic_adc.set_state(AdcPin::A0, AdcState::Enabled).unwrap();
        //generic_adc.set_state(AdcPin::A1, AdcState::Enabled);
        generic_adc.set_state(AdcPin::A2, AdcState::Enabled).unwrap();
        generic_adc.set_state(AdcPin::A3, AdcState::Enabled).unwrap();
        //generic_adc.set_state(AdcPin::A4, AdcState::Enabled);
        generic_adc.set_state(AdcPin::A5, AdcState::Enabled).unwrap();

        let readings = generic_adc.read_all().0;

        assert_eq!(readings[0], Ok(AdcReading::new::<12, u16>(0x00)));
        assert_eq!(
            readings[1],
            Err(AdcReadError((AdcPin::A1, AdcState::Disabled)))
        );
        assert_eq!(readings[2], Ok(AdcReading::new::<12, u16>(0xee)));
        assert_eq!(readings[3], Ok(AdcReading::new::<12, u16>(0x0d)));
        assert_eq!(
            readings[4],
            Err(AdcReadError((AdcPin::A4, AdcState::Disabled)))
        );
        assert_eq!(readings[5], Ok(AdcReading::new::<12, u16>(0xff)));
    }
}
