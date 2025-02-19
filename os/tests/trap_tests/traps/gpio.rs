use super::*;

use lc3_traits::peripherals::gpio::{Gpio, GpioPin, GpioState};

use GpioState::*;
use GpioPin::*;

mod states {
    use super::*;

    single_test! {
        input,
        insns: [
            { AND R0, R0, #0 },
            { TRAP #0x30 },
            { TRAP #0x25 },
        ],
        post: |i| {
            let p = i.get_peripherals();
            eq!(Gpio::get_state(p, G0), Input);
        },
        with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
    }

    single_test! {
        output,
        insns: [
            { AND R0, R0, #0 },
            { TRAP #0x31 },
            { TRAP #0x25 },
        ],
        post: |i| {
            let p = i.get_peripherals();
            eq!(Gpio::get_state(p, G0), Output);
        },
        with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
    }

    // TODO: TRAP x32 -- INTERRUPT (requires triggering Gpio interrupt externally)

    single_test! {
        disabled,
        insns: [
            { AND R0, R0, #0 },
            { TRAP #0x33 },
            { TRAP #0x25 },
        ],
        pre: |p| { Gpio::set_state(p, G0, Output).unwrap() },
        post: |i| {
            let p = i.get_peripherals();
            eq!(Gpio::get_state(p, G0), Disabled);
        },
        with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
    }

    single_test! {
        get_mode,
        prefill: { 0x3004: 0 },
        insns: [
            { AND R0, R0, #0 },
            { TRAP #0x34 },
            { ST R0, #1 },
            { TRAP #0x25 },
        ],
        pre: |p| { Gpio::set_state(p, G0, Output).unwrap() },
        post: |i| { eq!(i.get_word_unchecked(0x3004), 1); },
        with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
    }
}
