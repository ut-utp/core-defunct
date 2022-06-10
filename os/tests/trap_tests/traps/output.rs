use super::*;

// TODO: test for correct output.
// currently only testing that the machine halts.

// TODO: update these to use I/O peripherals now!

use lc3_os::traps::output as o;

// Print "!"
single_test! {
    out,
    prefill: { 0x3003: 33 },
    insns: [
        { LD R0, #2 },
        { TRAP #o::WRITE },
        { TRAP #HALT },
    ],
    with default os,
}

// Print "(!)"
single_test! {
    puts,
    prefill: {
        0x3003: 40,
        0x3004: 33,
        0x3005: 41,
        0x3006: 0
    },
    insns: [
        { LEA R0, #2 },
        { TRAP #PUTS },
        { TRAP #HALT },
    ],
    with default os,
}

// Print "(!)"
single_test! {
    putsp,
    prefill: {
        0x3003: 0x2128,
        0x3004: 0x0029,
        0x3006: 0
    },
    insns: [
        { LEA R0, #2 },
        { TRAP #PUTSP },
        { TRAP #HALT },
    ],
    with default os,
}
