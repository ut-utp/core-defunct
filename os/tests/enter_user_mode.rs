//! Tests that the OS calls user code correctly.
//!
//! Specifically the tests here check that the OS:
//!   - uses the user program start address set in
//!     [`USER_PROG_START_ADDR_SETTING_ADDR`](lc3_os::USER_PROG_START_ADDR_SETTING_ADDR)
//!   - only drops to user mode if that start address
//!     is below [`USER_PROGRAM_START_ADDR`](lc3_isa::USER_PROGRAM_START_ADDR)

extern crate lc3_test_infrastructure as lti;
use lti::{single_test, assert_eq as eq};

use lc3_baseline_sim::{interp::{InstructionInterpreter, InstructionInterpreterPeripheralAccess}, mem_mapped::PSR};
use lc3_os::{USER_PROG_START_ADDR_SETTING_ADDR, traps::builtin::*};
single_test! {
    respects_start_addr_setting,
    prefill: {
        0x4005: 0,
    },
    prefill_expr: {
        (USER_PROG_START_ADDR_SETTING_ADDR): 0x4000,
    },
    // The macro will actually go and set `USER_PROG_START_ADDR_SETTING_ADDR`
    // using this starting address but our `prefill` above overrides this.
    //
    // The idea here is to ensure that the OS is *not* simply using the default
    // starting address of `0x3000`. If it is it'll hit this HALT and won't
    // modify 0x4004.
    insns starting at { 0x3FFF }: [
        /* 0x3FFF */ { TRAP #HALT },
        /* 0x4000 */ { NOT R1, R0 },
        /* 0x4001 */ { AND R0, R1, R0 },
        /* 0x4002 */ { ADD R0, R0, #1 },
        /* 0x4003 */ { ST R0, #1 },
        /* 0x4004 */ { TRAP #HALT },
    ],
    memory: {
        0x4005: 1,
    },
    with default os,
}

// Rather than set `USER_PROG_START_ADDR_SETTING_ADDR` manually,
// these two tests rely on `single_test` doing it for us.
single_test! {
    does_not_drop_into_user_mode,
    insns starting at { 0x800 }: [
        /* Infinite Loop */
        { BRnzp #-1 },
    ],
    steps: 50_000, // Arbitrary; high enough to get past the OS.
    post: |i| {
        eq!(i.get_word_unchecked(USER_PROG_START_ADDR_SETTING_ADDR), 0x800);

        let psr = i.get_special_reg::<PSR>();
        assert!(psr.in_privileged_mode());
    },
    with default os,
}

single_test! {
    does_drop_into_user_mode,
    insns starting at { 0x5000 }: [
        /* Infinite Loop */
        { BRnzp #-1 },
    ],
    steps: 50_000, // Arbitrary; high enough to get past the OS.
    post: |i| {
        eq!(i.get_word_unchecked(USER_PROG_START_ADDR_SETTING_ADDR), 0x5000);

        let psr = i.get_special_reg::<PSR>();
        assert!(psr.in_user_mode());
    },
    with default os,
}
