use lc3_traits::peripherals::timers::*;
extern crate embedded_hal;
use embedded_hal as hal;
use embedded_hal::timer::*;//{Channel, OneShot};
use core::marker::PhantomData;
use core::cell::RefCell;
use core::sync::atomic::AtomicBool;


// //
// pub enum TimerType<S, T, U, V>
// where T: CountDown<Time = S>,
// 	  U: CountDown<Time = S> + Periodic
// {
// 	SingleShot(T),
// 	Repeated(U),
// }

//


//Using 2 timer generics here for the 2 timers we support to users. Both timers are marked periodic and periodic HAL timers
//are used even for singleshot mode. the reasoning is as follows
//THe HAL doesn't specify mechanisms to convert between oneshot and periodic modes. Hence we have 3 options here -
//1) Define custom trait extensions to add functionality for converting between timer types
//2) Specify 4 timers (2 independent singleshots, 2 independent periodic) in the generic timer unit and use them as necessary 
//   when the user configures in a certain mode. Since we support 2 timers, we have to have 2 independent timers of each mode
//   to avoid conversion between timers.
//3) Make both timer types periodic (has the marker trait Periodic in HAL) and just use the first countdown of the priodic timer
//   and ignore the rest if the user configures in Singleshot. The HAL also has support for a Cancel TRait and the periodic timer can be
//   cancelled/disabled after 1 period countdown.

//Chose to go with method 3 since method 2 requires more hardware support (4 timers instead of 2), method 1 requires custom trait
//extensions which needlessly makes the design cumbersome. Hence going with method 3.

//For method 3, there are a couple of ways to do the stopping of the timer in singleshot mode after one periodic countdown. Could
//either let the hardware timer continue running and just maintain a state array in software to indicate its disabled in which case the
//software doesn't check for interrupts from that timer (although the hardware timer interrupts will still be occuring periodically)/
//Another option is to use the cancel trait and cancel the countdown in periodic mode so the hardware timer doesn't generate any interruptd.
//The software timer state array should still be maintained and the timer state is set to disabled after the single shot countdown is done.
// The only difference is that using the cancel trait to cancel is more efficient for performance since the hardware is not generating useless
// interrupts (which can be a problem especially when the period is very small)
//TODO: Implement using cancel trait here. Also, implement cancel functionality for TM4C hal
pub struct generic_timer_unit<'a, S, T, U, V>
where T: CountDown<Time = V> + Periodic,
	  U: CountDown<Time = V> + Periodic,
      S: Into<u16> + From<u16> + Into<V>,
{ 
	t0: RefCell<T>,
	t1: RefCell<U>,
	states: RefCell<TimerArr<TimerState>>,
    modes: TimerArr<TimerMode>,
    interrupt_flags: Option<&'a TimerArr<AtomicBool>>,
    phantom: PhantomData<S>,
}

impl <'a, S, T, U, V> Default for generic_timer_unit<'a, S, T, U, V>
where T: CountDown<Time = V> + Periodic,
	  U: CountDown<Time = V> + Periodic,
      S: Into<u16> + From<u16> + Into<V>,
{
	fn default() -> Self{
		unimplemented!()
	}
}


//for initialization, the default is singleshot for both from trait spec. the actual types internally are periodic timers as described above,
//but again, it is exposed as singleshot to user



impl <'a, S, T, U, V> generic_timer_unit<'a, S, T, U, V>
where T: CountDown<Time = V> + Periodic,
	  U: CountDown<Time = V> + Periodic,
      S: Into<u16> + From<u16> + Into<V>,
{
	pub fn new(hal_timer0: T, hal_timer1: U) -> Self{

		Self{
			t0: RefCell::new(hal_timer0),
			t1: RefCell::new(hal_timer1),
            states: RefCell::new(TimerArr([TimerState::Disabled; TimerId::NUM_TIMERS])),
			modes: TimerArr([TimerMode::SingleShot; TimerId::NUM_TIMERS]),
            interrupt_flags: None,
            phantom: PhantomData,
		}
	}
}

macro_rules! timer_check_interrupt {
    ($hal_timer: ident, $timer_id: ident, $self: ident, $ret: ident, $states: ident) => {
        match $states[$timer_id]{
            TimerState::Disabled => {
                $ret = false;
            }
            TimerState::WithPeriod(_) => {
                match $hal_timer.wait() {
                    Ok(()) => {
                        $ret = true;
                        match $self.modes[$timer_id]{
                            TimerMode::SingleShot => {
                                $states[$timer_id] = TimerState::Disabled;
                            },
                            _ => {}
                        }
                        $self.interrupt_flags.unwrap()[$timer_id].store(true, core::sync::atomic::Ordering::SeqCst);// = AtomicBool::new(true); 
                    }
                    _=> {}
                }
            }
        }

    }
}

impl <'a, S, T, U, V> Timers<'a> for generic_timer_unit<'a, S, T, U, V>
where T: CountDown<Time = V> + Periodic,
	  U: CountDown<Time = V> + Periodic,
      S: Into<u16> + From<u16> + Into<V>,

 {
    fn set_mode(&mut self, timer: TimerId, mode: TimerMode) {
        self.modes[timer] = mode; 
    }
    fn get_mode(&self, timer: TimerId) -> TimerMode{TimerMode::SingleShot}
    #[inline]
    fn get_modes(&self) -> TimerArr<TimerMode> {
        let mut modes = TimerArr([TimerMode::SingleShot; TimerId::NUM_TIMERS]);

        TIMERS
            .iter()
            .for_each(|t| modes[*t] = self.get_mode(*t));

        modes
    }

    fn set_state(&mut self, timer: TimerId, state: TimerState) {
        self.states.borrow_mut()[timer] = state;

        match timer{
            TimerId::T0 => {
                match state {
                    TimerState::Disabled => {
                        //TODO: //Cancel trait function here
                    },
                    TimerState::WithPeriod(period) => {
                        self.t0.borrow_mut().start(S::from(core::num::NonZeroU16::get(period)));
                    },
                }
            }

            TimerId::T1 => {
                match state {
                    TimerState::Disabled => {
                        //TODO: //Cancel trait function here
                    },
                    TimerState::WithPeriod(period) => {
                        self.t1.borrow_mut().start(S::from(core::num::NonZeroU16::get(period)));
                    },
                }
            }
        }
    }
    fn get_state(&self, timer: TimerId) -> TimerState {
        self.states.borrow_mut()[timer]
    }
    #[inline]
    fn get_states(&self) -> TimerArr<TimerState> {
        let mut states = TimerArr([TimerState::Disabled; TimerId::NUM_TIMERS]);

        TIMERS
            .iter()
            .for_each(|t| states[*t] = self.get_state(*t));

        states
    }

    fn register_interrupt_flags(&mut self, flags: &'a TimerArr<AtomicBool>) {
        self.interrupt_flags = Some(flags);
    }

    fn interrupt_occurred(&self, timer: TimerId) -> bool {
        let mut ret = false;
        let mut t0 = self.t0.borrow_mut();
        let mut t1 = self.t1.borrow_mut();
        let mut states = self.states.borrow_mut();

        match timer{
            TimerId::T0 => {
                timer_check_interrupt!(t0, timer, self, ret, states);
            },
            TimerId::T1 => {
                timer_check_interrupt!(t1, timer, self, ret, states);
            }
        }

        // match self.states[timer]{
        //     TimerState::Disabled => {
        //         ret = false;
        //     }
        //     TimerState::WithPeriod(_) => {
        //         match self.t0.borrow_mut().wait() {
        //             Ok(()) => {
        //                 ret = true;
        //                 match self.modes[timer]{
        //                     TimerMode::SingleShot => {
        //                         self.states[timer] = TimerState::Disabled;
        //                     },
        //                     _ => {}
        //                 }
        //             }
        //             _=> {}
        //         }
        //     }
        // }
        
        ret
    }
    fn reset_interrupt_flag(&mut self, timer: TimerId) {
        self.interrupt_flags.unwrap()[timer].store(false, core::sync::atomic::Ordering::SeqCst);
    }
    #[inline]
    fn interrupts_enabled(&self, timer: TimerId) -> bool {
        matches!(self.get_state(timer), TimerState::WithPeriod(_)) ||
        (self.get_state(timer) == TimerState::Disabled && self.interrupt_occurred(timer))
    }
 }