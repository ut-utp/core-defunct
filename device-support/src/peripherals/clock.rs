use lc3_isa::Word;
use lc3_traits::peripherals::clock::Clock;

use embedded_time::{
    duration::{Generic, Milliseconds},
    fixed_point::FixedPoint,
    Clock as HalClock,
};
use num_traits::{AsPrimitive, WrappingSub};

// Note to implementors: we reccomend that your clock have a period that's
// a multiple of `65536` (aka 2^16) so it doesn't overflow to 0 at seemingly
// random times within the simulator.
//
// We assume the clock counts up.
pub struct GenericClock<C>
where
    C: HalClock,
    C::T: AsPrimitive<Word>,
{
    clock: C,
    base_ref: Generic<C::T>,
}

impl<C: HalClock> GenericClock<C>
where
    C::T: AsPrimitive<Word> + From<Word>,
{
    pub fn new(hal_clock: C) -> Self {
        Self {
            base_ref: hal_clock.try_now().unwrap().duration_since_epoch(),
            clock: hal_clock,
        }
    }
}

impl<C: HalClock> Clock for GenericClock<C>
where
    C::T: AsPrimitive<Word> + From<Word>,
{
    //just unwrap since the utp clock trait is currently infallible
    //can't really do anything on error. so will just crash on clock error
    fn get_milliseconds(&self) -> Word {
        let now = self.clock.try_now().unwrap();

        // We specifically want to use wrapping sub here but _after_ the
        // conversion to milliseconds (which is why we store `base_ref` as a
        // `Generic` and not in milliseconds).
        let ticks_now = now.duration_since_epoch();
        let ticks_elapsed =
            ticks_now.integer().wrapping_sub(&self.base_ref.integer());

        // Convert back into `Generic` to calculate the number of milliseconds:
        let ticks_elapsed =
            Generic::new(ticks_elapsed, *ticks_now.scaling_factor());
        let millis_elapsed: Milliseconds<C::T> =
            ticks_elapsed.try_into().unwrap();
        let millis_elapsed = millis_elapsed.integer();

        // Bound to [0, u16::MAX]:
        let millis_elapsed =
            millis_elapsed % From::<u32>::from((Word::MAX as u32) + 1);
        millis_elapsed.as_()
    }

    fn set_milliseconds(&mut self, ms: Word) {
        let ms = Milliseconds::<C::T>::new(ms.into());
        let ms: Generic<C::T> = ms.into();

        let now = self.clock.try_now().unwrap().duration_since_epoch();

        let ms_ticks = ms.integer();
        let now_ticks = ms.integer();

        let base_ticks = now_ticks.wrapping_sub(&ms_ticks);

        self.base_ref = Generic::new(base_ticks, *now.scaling_factor());
    }
}

// TODO: tests using `std-embedded-time`
