use lc3_traits::peripherals::adc::*;
extern crate embedded_hal;
use embedded_hal as hal;
use embedded_hal::adc::{Channel, OneShot};
use core::marker::PhantomData;
use core::cell::RefCell;

pub struct Adc1;

pub struct generic_adc_unit<T, U, WORD, ADC>
where T: hal::adc::Channel<ADC>,
	  U: hal::adc::OneShot<ADC, WORD, T>,
	  WORD: From<u16>,
{ 
	//x: T
	hal_adc: RefCell<U>,
	hal_pins: RefCell<AdcPinArr<T>>,
	pin_states: AdcPinArr<AdcState>,
	phantom: PhantomData<WORD>,
	phantom2: PhantomData<ADC>,

}

impl <T, U, WORD, ADC> Default for generic_adc_unit<T, U, WORD, ADC>
where T: hal::adc::Channel<ADC>,
	  U: hal::adc::OneShot<ADC, WORD, T>,
	  WORD: From<u16>,
{
	fn default() -> Self{
		unimplemented!()
	}
}

impl <T, U, WORD, ADC> generic_adc_unit<T, U, WORD, ADC>
where T: hal::adc::Channel<ADC>,
	  U: hal::adc::OneShot<ADC, WORD, T>,
	  WORD: From<u16>,
{
	fn new(hal_adc: U, pins: AdcPinArr<T>) -> Self{
		Self{
			hal_adc: RefCell::new(hal_adc),
			hal_pins: RefCell::new(pins),
			pin_states: AdcPinArr([AdcState::Disabled; AdcPin::NUM_PINS]),
			phantom: PhantomData,
			phantom2: PhantomData,
		}
	}
}

impl <T, U, WORD, ADC> Adc for generic_adc_unit<T, U, WORD, ADC>
where T: hal::adc::Channel<ADC>,
	  U: hal::adc::OneShot<ADC, WORD, T>,
	  WORD: From<u16> + Into<u16>, // This trait bound is the only additional constraint being imposed on the HAL generics
	  								//to make them usable with our traits. This is necessary since the HAL trait definition 
	  								//does not impose any bounds on WORD but we need to get an integer from it to use it with our platform
	  								//Hence, this might require one small additional custom board specific implementation to the HAL traits to convert
	  								//the WORD type they use to/from u16 (if the board specific hal impl doesn't already implement it for its WORD). 
	  								//u16 was picked as it seems like a reasonable type for adc readings, and the embedded-hal
	  								//official example also uses u16 for adc reading.
{
	//HAL has no notion of state/enabled or disabled pins. So just maintaining the pin state in as a pin array software
	//Therefore, Users will still have to enable pins to use them, but this implementation will just return Ok(()) always
	//since no operation is done on the actual adc peripheral. just infallible software operations
    fn set_state(&mut self, pin: AdcPin, state: AdcState) -> Result<(), AdcMiscError>{
    	//let mut pins = self.pin_.borrow_mut();
    	self.pin_states[pin] = state;
    	Ok(())
    }
    fn get_state(&self, pin: AdcPin) -> AdcState{
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

    fn read(&self, pin: AdcPin) -> Result<u8, AdcReadError>{
    	let mut adc_unit = self.hal_adc.borrow_mut();
    	let mut pins = self.hal_pins.borrow_mut();
    	
    	let mut adc_reading: Result<u8, AdcReadError> = Err(AdcReadError((pin, AdcState::Disabled)));
    	let mut value_debug = 0;

    	if(self.get_state(pin) == AdcState::Enabled){

    		let result = adc_unit.read(&mut pins[pin]);

	    	match result{
	    		Ok(value) => {
	    			//value_debug = value.into();
	    			adc_reading = Ok((value.into() >> 1) as u8)
	    		},
	    		_ => {
	    			//adc_reading = Err(AdcReadError((pin, AdcState::Disabled))) would return this error
	    			// TODO: This is not the correct eror type. should be miscallaneous error for HAL read fail?
	    		},
	    	}
	    }

    	adc_reading
    }
    #[inline]
    fn read_all(&self) -> AdcPinArr<Result<u8, AdcReadError>> {
        // TODO: Error conversion impl (see Gpio)
        let mut readings = AdcPinArr([Ok(0u8); AdcPin::NUM_PINS]); // TODO: that we need a default value here is weird and bad...

        ADC_PINS
            .iter()
            .for_each(|a| readings[*a] = self.read(*a));

        readings
    }

}

