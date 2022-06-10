use super::*;

use lc3_baseline_sim::mem_mapped::{
    MemMapped,
    T0CR_ADDR, T0DR_ADDR,
    TIMER_BASE_INT_VEC,
    PSR,
    MCR
};

single_test! {
    singleshot_10ms,
    prefill: {
        0x3010: 0x0700,
        0x3011: T0CR_ADDR,
        0x3012: T0DR_ADDR,
        0x3013: 10,
        0x3014: <MCR as MemMapped>::ADDR,
        0x3015: 0,
    },
    prefill_expr: {
        (TIMER_BASE_INT_VEC): 0x3009,
        (<PSR as MemMapped>::ADDR): 0x0302,
    },
    insns: [
        { LD R6, #0xF },    // Set nonzero R6
        { AND R0, R0, #0 }, // Mode: singleshot
        { STI R0, #0xE },   // Set to singleshot
        { LD R0, #0xF },    // Load period (10ms)
        { STI R0, #0xD },   // Set period to 10ms
        { LD R0, #0xF },    // Check if interrupt fired
        { BRnz #-2 },       // Go back one if not set

        { AND R0, R0, #0 }, // Prep HALT
        { STI R0, #0xB },   // HALT (0x3008)

        { AND R1, R1, #0 },
        { ADD R1, R1, #1 }, // R1 <- #1
        { ST R1, #9 },
        { RTI } // 0x300C
    ],
}

// TODO: flaky (sometimes hangs)
single_test! {
    repeated_10ms,
    prefill: {
        0x3010: 0x0700,
        0x3011: T0CR_ADDR,
        0x3012: T0DR_ADDR,
        0x3013: 10,
        0x3014: <MCR as MemMapped>::ADDR,
        0x3015: 0xFFFC, // -4
    },
    prefill_expr: {
        (TIMER_BASE_INT_VEC): 0x3009,
        (<PSR as MemMapped>::ADDR): 0x0302, // set starting priority to 3 (below
                                            // the timer interrupt's priority: 4)
    },
    insns: [
        /* 0x3000 */ { LD R6, #0xF },    // Set nonzero R6

        /* 0x3001 */ { AND R0, R0, #0 },
        /* 0x3002 */ { ADD R0, R0, #1 }, // Mode: repeated
        /* 0x3003 */ { STI R0, #0xD },   // Set to repeated

        /* 0x3004 */ { LD R0, #0xE },    // Load period (10ms)
        /* 0x3005 */ { STI R0, #0xC },   // Set period to 10ms

        /* 0x3006 */ { LD R0, #0xE },    // Check if interrupt fired five times
        /* 0x3007 */ { BRnz #-2 },       // Go back one if not

        /* 0x3008 */ { AND R0, R0, #0 }, // Prep HALT
        /* 0x3009 */ { STI R0, #0xA },   // HALT

        /* 0x300A */ { LD R1, #0xA },
        /* 0x300B */ { ADD R1, R1, #1 },
        /* 0x300C */ { ST R1, #0x8 },
        /* 0x300D */ { RTI }
    ],
}

// TODO: support labels in the `single_test`, etc. macros
// TODO: use `tracing`, especially in the interpreter
// TODO: use proptest for these tests instead of just sweeping the entire possibility space
