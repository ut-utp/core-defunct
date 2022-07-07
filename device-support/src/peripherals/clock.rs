use lc3_traits::peripherals::clock::*;
use lc3_isa::Word;

extern crate embedded_time;
use embedded_time as hal_time;
use hal_time::fraction::Fraction;
use hal_time::TimeInt;
use hal_time::duration::{Milliseconds, Generic};
use hal_time::fixed_point::FixedPoint;

use core::marker::PhantomData;
use core::cell::RefCell;
use core::hash::Hash;
use core::convert::TryFrom;

//Clock::T should have been generic U rather than u64.
//But it's a hassle to connect across board device impl module and this
//due to the need to convert u64 to u16 and defining custom connecting types
//to satisfy TimeInt, Hash and Into<u16>.
//TODO: Revisit this later if needed 
pub struct generic_clock_unit<T, U>
where T: hal_time::clock::Clock<T = u64>, 
	 // U: TimeInt + Hash + Into<u16>,
{
	clock_unit: T,
	base_ref: u16,
	phantom: PhantomData<U>,
}

impl <T, U> Default for generic_clock_unit<T, U>
where T: hal_time::clock::Clock<T = u64>,
	  //U: TimeInt + Hash + Into<u16>,
{
	fn default() -> Self{
		unimplemented!()
	}
}

impl <T, U> generic_clock_unit<T, U>
where T: hal_time::clock::Clock<T = u64>,
	  //U: TimeInt + Hash + Into<u16>,
{
	pub fn new(hal_clock: T) -> Self{
		Self{
			clock_unit: hal_clock,
			base_ref: 0,
			phantom: PhantomData,
		}
	}
}

impl <T, U> Clock for generic_clock_unit<T, U>
where T: hal_time::clock::Clock<T = u64>,
	  //U: TimeInt + Hash + Into<u16>,
{
	//just unwrap since the utp clock trait is currently infallible
	//can't really do anything on error. so will just crash on clock error
    fn get_milliseconds(&self) -> Word {
    	let instant = self.clock_unit.try_now().unwrap(); 
    	let generic_duration: Generic<u64> = instant.duration_since_epoch(); //duration since start
    	//let hal_milliseconds = U::try_int//generic_duration.0;
    	let hal_milliseconds = Milliseconds::<u64>::try_from(generic_duration).unwrap();
    	let milliseconds = hal_milliseconds.integer();
    	((milliseconds & 0xFFFF) as u16).wrapping_add(self.base_ref)
    }

    fn set_milliseconds(&mut self, ms: Word) {
    	self.base_ref = ms;
    }
}