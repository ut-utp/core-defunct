use super::*;

use lc3_baseline_sim::mem_mapped::CLKR_ADDR;
use std::thread::sleep;
use std::time::Duration;

const TOLERANCE: u16 = 5;

single_test! {
    get_immediately,
    prefill: { 0x3010: CLKR_ADDR },
    insns: [ { LDI R0, #0xF } ],
    steps: 1,
    post: |i| { assert_is_about(i.get_register(R0), 0, TOLERANCE); }
}

single_test! {
    get_after_100ms,
    prefill: { 0x3010: CLKR_ADDR },
    insns: [ { LDI R0, #0xF } ],
    steps: 1,
    pre: |_| { sleep(Duration::from_millis(100)); },
    post: |i| { assert_is_about(i.get_register(R0), 100, TOLERANCE); }
}

single_test! {
    reset,
    prefill: { 0x3010: CLKR_ADDR },
    insns: [
        { AND R0, R0, #0 },
        { STI R0, #0xE },
        { LDI R0, #0xD },
    ],
    steps: 3,
    pre: |_| { sleep(Duration::from_millis(100)); },
    post: |i| { assert_is_about(i.get_register(R0), 0, TOLERANCE); }
}

single_test! {
    set_higher,
    prefill: {
        0x3010: CLKR_ADDR,
        0x3011: 1000,
    },
    insns: [
        { LD R0, #0x10 },
        { STI R0, #0xE },
        { LDI R0, #0xD }
    ],
    steps: 3,
    pre: |_| { sleep(Duration::from_millis(100)); },
    post: |i| { assert_is_about(i.get_register(R0), 1000, TOLERANCE); }
}
