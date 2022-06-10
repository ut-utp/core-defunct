use super::*;

use lc3_traits::peripherals::adc::{Adc, AdcPin, AdcState};
use lc3_shims::peripherals::AdcShim;

use AdcState::*;
use AdcPin::*;

use std::sync::RwLock;

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
    pre: |p| { Adc::set_state(p, A0, Enabled); },
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
    pre: |p| { Adc::set_state(p, A0, Disabled); },
    post: |i| { eq!(i.get_word_unchecked(0x3004), 0); },
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
    pre: |p| { Adc::set_state(p, A0, Enabled); },
    post: |i| { eq!(i.get_word_unchecked(0x3004), 1); },
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
        Adc::set_state(p, A0, Enabled);
        AdcShim::set_value(&mut *p.get_adc().write().unwrap(), A0, 10);
    },
    post: |i| { eq!(i.get_word_unchecked(0x3004), 10); },
    with default os,
}
