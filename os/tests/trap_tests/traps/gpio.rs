use super::*;

use lc3_traits::peripherals::gpio::{Gpio, GpioPin, GpioState, GPIO_PINS};
use lc3_baseline_sim::mem_mapped::{
    G0_INT_VEC, G1_INT_VEC, G2_INT_VEC, G3_INT_VEC,
    G4_INT_VEC, G5_INT_VEC, G6_INT_VEC, G7_INT_VEC,
};

use GpioState::*;
use GpioPin::*;

use lc3_os::traps::gpio as g;

mod states {
    use super::*;

    single_test! {
        input,
        insns: [
            { AND R0, R0, #0 },
            { TRAP #g::INPUT },
            { TRAP #HALT },
        ],
        post: |i| {
            let p = i.get_peripherals();
            eq!(Gpio::get_state(p, G0), Input);
        },
        with default os,
    }

    single_test! {
        output,
        insns: [
            { AND R0, R0, #0 },
            { TRAP #g::OUTPUT },
            { TRAP #HALT },
        ],
        post: |i| {
            let p = i.get_peripherals();
            eq!(Gpio::get_state(p, G0), Output);
        },
        with default os,
    }

    // TODO: TRAP x32 -- INTERRUPT (requires triggering Gpio interrupt externally)

    single_test! {
        disabled,
        insns: [
            { AND R0, R0, #0 },
            { TRAP #g::DISABLED },
            { TRAP #HALT },
        ],
        pre: |p| { Gpio::set_state(p, G0, Output); },
        post: |i| {
            let p = i.get_peripherals();
            eq!(Gpio::get_state(p, G0), Disabled);
        },
        with default os,
    }

    single_test! {
        get_mode,
        prefill: { 0x3004: 0 },
        insns: [
            { AND R0, R0, #0 },
            { TRAP #g::GET_MODE },
            { ST R0, #1 },
            { TRAP #HALT },
        ],
        pre: |p| { Gpio::set_state(p, G0, Output); },
        post: |i| { eq!(i.get_word_unchecked(0x3004), 1); },
        with default os,
    }

    // TODO: TRAP x35 (write)
    // TODO: TRAP x36 (read)
}
