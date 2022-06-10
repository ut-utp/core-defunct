use super::*;

use lc3_traits::peripherals::{Peripherals, adc::{Adc, AdcPin, AdcState}};
use lc3_shims::peripherals::AdcShim;

use AdcState::*;
use AdcPin::*;
use lc3_os::traps::adc as a;

single_test! {
    enable,
    insns: [
        { AND R0, R0, #0 },
        { TRAP #a::ENABLE },
        { TRAP #HALT },
    ],
    post: |i| {
        let p = i.get_peripherals();
        eq!(Adc::get_state(p, A0), Enabled);
    },
    with default os,
}

single_test! {
    disable,
    insns: [
        { AND R0, R0, #0 },
        { TRAP #a::DISABLE },
        { TRAP #HALT },
    ],
    pre: |p| { Adc::set_state(p, A0, Enabled).unwrap() },
    post: |i| {
        let p = i.get_peripherals();
        eq!(Adc::get_state(p, A0), Disabled);
    },
    with default os,
}

single_test! {
    get_mode,
    prefill: { 0x3004: 0 },
    insns: [
        { AND R0, R0, #0 },
        { TRAP #a::GET_MODE },
        { ST R0, #1 },
        { TRAP #HALT },
    ],
    pre: |p| { Adc::set_state(p, A0, Disabled).unwrap() },
    post: |i| { eq!(i.get_word_unchecked(0x3004), 0); },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
    with default os,
}

single_test! {
    get_mode_enabled,
    prefill: { 0x3004: 0 },
    insns: [
        { AND R0, R0, #0 },
        { TRAP #a::GET_MODE },
        { ST R0, #1 },
        { TRAP #HALT },
    ],
    pre: |p| { Adc::set_state(p, A0, Enabled).unwrap() },
    post: |i| { eq!(i.get_word_unchecked(0x3004), 1); },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
    with default os,
}

single_test! {
    read,
    prefill: { 0x3004: 0 },
    insns: [
        { AND R0, R0, #0 },
        { TRAP #a::READ },
        { ST R0, #1 },
        { TRAP #HALT },
    ],
    pre: |p| {
        Adc::set_state(p, A0, Enabled).unwrap();
        AdcShim::set_value(&mut *p.get_adc().write().unwrap(), A0, 10).unwrap();
    },
    post: |i| { eq!(i.get_word_unchecked(0x3004), 10); },
    with default os,
}
