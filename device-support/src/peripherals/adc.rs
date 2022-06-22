use lc3_traits::peripherals::adc::*;
extern crate embedded_hal;
use embedded_hal as hal;
use embedded_hal::adc::{Channel, OneShot};
use core::marker::PhantomData;
use core::cell::RefCell;

pub struct Adc1;

macro_rules! ambiguity {
    ($($i:ident)* $j:ident) => { };
}

pub struct generic_adc_unit<U0, U1, U2, U3, U4, U5, A0, A1, A2, A3, A4, A5, WORD, ADC>
where
	  A0: Channel<ADC>,
	  A1: Channel<ADC>,
	  A2: Channel<ADC>,
	  A3: Channel<ADC>,
	  A4: Channel<ADC>,
	  A5: Channel<ADC>,
	  U0: OneShot<ADC, WORD, A0>,
	  U1: OneShot<ADC, WORD, A1>,
	  U2: OneShot<ADC, WORD, A2>,
	  U3: OneShot<ADC, WORD, A3>,
	  U4: OneShot<ADC, WORD, A4>,
	  U5: OneShot<ADC, WORD, A5>,
	  WORD: From<u16> +Into<u16> ,
{ 
	port0: RefCell<U0>,
	port1: RefCell<U1>,
	port2: RefCell<U2>,
	port3: RefCell<U3>,
	port4: RefCell<U4>,
	port5: RefCell<U5>,
	a0: RefCell<A0>,
	a1: RefCell<A1>,
	a2: RefCell<A2>,
	a3: RefCell<A3>,
	a4: RefCell<A4>,
	a5: RefCell<A5>,
	pin_states: AdcPinArr<AdcState>,
	phantom: PhantomData<WORD>,
	phantom2: PhantomData<ADC>,
	phantom3: PhantomData<U0>,

}

impl <U0, U1, U2, U3, U4, U5, A0, A1, A2, A3, A4, A5, WORD, ADC> Default for generic_adc_unit<U0, U1, U2, U3, U4, U5, A0, A1, A2, A3, A4, A5, WORD, ADC>
where
	  A0: Channel<ADC>,
	  A1: Channel<ADC>,
	  A2: Channel<ADC>,
	  A3: Channel<ADC>,
	  A4: Channel<ADC>,
	  A5: Channel<ADC>,
	  U0: OneShot<ADC, WORD, A0>,
	  U1: OneShot<ADC, WORD, A1>,
	  U2: OneShot<ADC, WORD, A2>,
	  U3: OneShot<ADC, WORD, A3>,
	  U4: OneShot<ADC, WORD, A4>,
	  U5: OneShot<ADC, WORD, A5>,
	  WORD: From<u16> +Into<u16> ,
{
	fn default() -> Self{
		unimplemented!()
	}
}

impl <U0, U1, U2, U3, U4, U5, A0, A1, A2, A3, A4, A5, WORD, ADC> generic_adc_unit<U0, U1, U2, U3, U4, U5, A0, A1, A2, A3, A4, A5, WORD, ADC>
where
	  A0: Channel<ADC>,
	  A1: Channel<ADC>,
	  A2: Channel<ADC>,
	  A3: Channel<ADC>,
	  A4: Channel<ADC>,
	  A5: Channel<ADC>,
	  U0: OneShot<ADC, WORD, A0>,
	  U1: OneShot<ADC, WORD, A1>,
	  U2: OneShot<ADC, WORD, A2>,
	  U3: OneShot<ADC, WORD, A3>,
	  U4: OneShot<ADC, WORD, A4>,
	  U5: OneShot<ADC, WORD, A5>,
	  WORD: From<u16> +Into<u16> ,
{
	fn new(port0: U0, port1: U1, port2: U2, port3: U3, port4: U4, port5: U5, p0: A0, p1: A1, p2: A2, p3: A3, p4: A4, p5: A5) -> Self{
		Self{
			port0: RefCell::new(port0),
			port1: RefCell::new(port1),
			port2: RefCell::new(port2),
			port3: RefCell::new(port3),
			port4: RefCell::new(port4),
			port5: RefCell::new(port5),
			a0: RefCell::new(p0),
			a1: RefCell::new(p1),
			a2: RefCell::new(p2),
			a3: RefCell::new(p3),
			a4: RefCell::new(p4),
			a5: RefCell::new(p5),
			pin_states: AdcPinArr([AdcState::Disabled; AdcPin::NUM_PINS]),
			phantom: PhantomData,
			phantom2: PhantomData,
			phantom3: PhantomData,
		}
	}
}


macro_rules! adc_read_pin {
	($pin: ident, $port: ident, $self: ident, $adc_reading: ident) => {
			 		let mut pin = $self.$pin.borrow_mut();
			 		let mut port = $self.$port.borrow_mut();
					let result = port.read(&mut *pin);

			    	match result{
			    		Ok(value) => {
			    			//value_debug = value.into();
			    			$adc_reading = Ok((value.into() >> 1) as u8)
			    		},
			    		_ => {
			    			//adc_reading = Err(AdcReadError((pin, AdcState::Disabled))) would return this error
			    			// TODO: This is not the correct eror type. should be miscallaneous error for HAL read fail?
			    		},
			    	}

	}
}

impl <U0, U1, U2, U3, U4, U5, A0, A1, A2, A3, A4, A5, WORD, ADC> Adc for generic_adc_unit<U0, U1, U2, U3, U4, U5, A0, A1, A2, A3, A4, A5, WORD, ADC>
where
	  A0: Channel<ADC>,
	  A1: Channel<ADC>,
	  A2: Channel<ADC>,
	  A3: Channel<ADC>,
	  A4: Channel<ADC>,
	  A5: Channel<ADC>,
	  U0: OneShot<ADC, WORD, A0>,
	  U1: OneShot<ADC, WORD, A1>,
	  U2: OneShot<ADC, WORD, A2>,
	  U3: OneShot<ADC, WORD, A3>,
	  U4: OneShot<ADC, WORD, A4>,
	  U5: OneShot<ADC, WORD, A5>,
	  WORD: From<u16> +Into<u16> , // This trait bound is the only additional constraint being imposed on the HAL generics
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
    	//let mut adc_unit = self.hal_adc.borrow_mut();
    	//let mut pins = self.hal_pins.borrow_mut();
    	
    	let mut adc_reading: Result<u8, AdcReadError> = Err(AdcReadError((pin, AdcState::Disabled)));
    	let mut value_debug = 0;

    	if(self.get_state(pin) == AdcState::Enabled){

    		match pin {
    			AdcPin::A0 =>{
    				adc_read_pin!(a0, port0, self, adc_reading);
    			},
    			AdcPin::A1 => {
    				adc_read_pin!(a1, port1, self, adc_reading);
    			}
    			AdcPin::A2 =>{
    				adc_read_pin!(a2, port2, self, adc_reading);
    			},
    			AdcPin::A3 => {
    				adc_read_pin!(a3, port3, self, adc_reading);
    			}
    			AdcPin::A4 =>{
    				adc_read_pin!(a4, port4, self, adc_reading);
    			},
    			AdcPin::A5 => {
    				adc_read_pin!(a5, port5, self, adc_reading);
    			}
    			_ =>{},
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

