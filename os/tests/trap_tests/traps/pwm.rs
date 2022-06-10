use super::*;

use core::num::NonZeroU8;
use lc3_traits::peripherals::pwm::{Pwm, PwmPin, PwmState, PwmDutyCycle};
use PwmState::*;
use PwmPin::*;

use lc3_os::traps::pwm as p;

single_test! {
    enable,
    prefill: {
        0x3005: 20,
        0x3006: 128
    },
    insns: [
        { AND R0, R0, #0 },
        { LD R1, #3 },
        { LD R2, #3 },
        { TRAP #p::ENABLE },
        { TRAP #HALT },
    ],
    post: |i| {
        let p = i.get_peripherals();
        eq!(Pwm::get_state(p, P0), Enabled(NonZeroU8::new(20).unwrap()));
        eq!(Pwm::get_duty_cycle(p, P0), 128);
    },
    with default os,
}

single_test! {
    disable,
    insns: [
        { AND R0, R0, #0 },
        { TRAP #p::DISABLE },
        { TRAP #HALT },
    ],
    pre: |p| {
        Pwm::set_state(p, P0, Enabled(NonZeroU8::new(20).unwrap()));
        Pwm::set_duty_cycle(p, P0, 128);
    },
    post: |i| {
        let p = i.get_peripherals();
        eq!(Pwm::get_state(p, P0), Disabled);
        eq!(Pwm::get_duty_cycle(p, P0), 128);
    },
    with default os,
}

single_test! {
    get_period,
    prefill: { 0x3004: 0 },
    insns: [
        { AND R0, R0, #0 },
        { TRAP #p::GET_PERIOD },
        { ST R0, #1 },
        { TRAP #HALT },
    ],
    pre: |p| {
        Pwm::set_state(p, P0, Enabled(NonZeroU8::new(20).unwrap()));
        Pwm::set_duty_cycle(p, P0, 128);
    },
    post: |i| {
        eq!(i.get_word_unchecked(0x3004), 20);
    },
    with default os,
}

single_test! {
    get_period_disabled,
    prefill: { 0x3004: 0xBEEF },
    insns: [
        { AND R0, R0, #0 },
        { TRAP #p::GET_PERIOD },
        { ST R0, #1 },
        { TRAP #HALT },
    ],
    post: |i| {
        eq!(i.get_word_unchecked(0x3004), 0);
    },
    with default os,
}

single_test! {
    get_period_invalid_pin,
    prefill: { 0x3005: 0xBEEF },
    insns: [
        { AND R0, R0, #0 },
        { ADD R0, R0, #10 },
        { TRAP #p::GET_PERIOD },
        { ST R0, #1 },
        { TRAP #HALT },
    ],
    post: |i| {
        eq!(i.get_word_unchecked(0x3005), 0);
    },
    with default os,
}

single_test! {
    get_duty,
    prefill: { 0x3004: 0 },
    insns: [
        { AND R0, R0, #0 },
        { TRAP #p::GET_DUTY },
        { ST R0, #1 },
        { TRAP #HALT },
    ],
    pre: |p| {
        Pwm::set_state(p, P0, Enabled(NonZeroU8::new(20).unwrap()));
        Pwm::set_duty_cycle(p, P0, 128);
    },
    post: |i| {
        eq!(i.get_word_unchecked(0x3004), 128);
    },
    with default os,
}

// COMMIT MSG: change the `single_test` infra to
// actually run OS init; this tests that trap tests (and other
// tests that run with an OS) run correctly in _user mode_

// COMMIT MSG: allow setting the starting address for the instruction stream
// in the macro; this lets us make use of the OS' "drop to user mode
// only if at or above 0x3000" in the test macro
//
//  - include: test
