use super::*;

use std::thread::sleep;
use std::time::Duration;
use super::lti::single_test;
use lc3_traits::peripherals::Clock;

const TOLERANCE: u16 = 5;

single_test! {
    set,
    insns: [
        { AND R0, R0, #0 },
        { TRAP #0x70 },
        { TRAP #0x25 },
    ],
    pre: |_| { sleep(Duration::from_millis(100)); },
    post: |i| { assert_is_about(Clock::get_milliseconds(i.get_peripherals()), 0, TOLERANCE); },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}

single_test! {
    get,
    prefill: { 0x3003: 0 },
    insns: [
        { TRAP #0x71 },
        { ST R0, #1 },
        { TRAP #0x25 },
    ],
    pre: |_| { sleep(Duration::from_millis(200)); },
    post: |i| { assert_is_about(i.get_word_unchecked(0x3003), 200, TOLERANCE); },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}

