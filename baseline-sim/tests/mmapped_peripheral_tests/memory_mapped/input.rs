use super::*;

use lc3_baseline_sim::mem_mapped::{KBSR_ADDR, KBDR_ADDR, KEYBOARD_INT_VEC};
use lc3_isa::{PSR, INTERRUPT_VECTOR_TABLE_START_ADDR};


// TODO: test that we drop new chars instead of retaining the last char..
// TODO: test that interrupts keep triggering until we read KBDR, clearing
// the ready bit

single_test! {
    interrupts_will_continue_until_morale_improves,
    prefill_expr: {
        // Register our ISR as the Keyboard Interrupt Handler:
        (INTERRUPT_VECTOR_TABLE_START_ADDR + (KEYBOARD_INT_VEC as Word)): 0x20F,

        // Mask for bit 14:
        (0x200): !0b0100_0000_0000_0000,
        // KBSR address:
        (0x201): KBSR_ADDR,
        // PSR address:
        (0x202): PSR,
        // PSR value:
        (0x203): 0b1_0000_111_00000_0_1_0,
        //         |       |        | | \
        //         |       |        | \  p bit
        //         |       |        \  z bit
        //         |       \        n bit
        //         \   priority level (3)
        //   supervisor mode

        // ISR Remaining:
        (0x220): 5,
        // ISR Count:
        (0x221): 0,
        // KBDR address:
        (0x222): KBDR_ADDR,
    },
    // Because we're running with no OS nothing is there to move us
    // out of supervisor mode; we don't have to worry about ACVs.
    insns starting at { 0x204 }: [
        // Enable interrupts (bit 14 of KBSR):
        /* 0x204 */ { LD R2, #-5 },     // R2 := mask
        /* 0x205 */ { LD R0, #-5 },     // R0 := KBSR_ADDR
        /* 0x206 */ { LD R3, #-5 },     // R3 := PSR_ADDR
        /* 0x207 */ { LD R4, #-5 },     // R4 := PSR value
        /* 0x208 */ { LDR R1, R0, #0 }, // R1 := KBSR value

        // Mask out bit 14, set it:
        /* 0x209 */ { AND R1, R1, R2 },
        /* 0x20A */ { NOT R2, R2 },
        /* 0x20B */ { ADD R1, R1, R2 },

        // Store it:
        /* 0x20C */ { STR R1, R0, #0 },

        // Set the PSR (to lower the current priority so that interrupts can
        // actually run):
        /* 0x20D */ { STR R4, R3, #0 },

        // Spin:
        /* 0x20E */ { BRnzp #0 },

        /////////////// ISR ///////////////
        // Playing it fast and loose with registers here instead of using the
        // stack because we *know* there's no user code.

        // The idea is to have the ISR only read `KBDR` once `ISR_REMAINING`
        // hits zero.
        //
        // The ISR also unconditionally increments `ISR_COUNT`.
        //
        // This lets us ensure both that a single input character, if not read
        // in the ISR, will cause the ISR to be triggered repeatedly
        // (`ISR_REMAINING` being less than 5, the starting value, tells us this
        // is working) *and* that once `KBDR` is read the ISRs will stop
        // (`ISR_COUNT` being 6 and not some higher value tells us this is
        // working).
        /* 0x20F */ { LEA R0, #16 },

        // Increment ISR Count:
        /* 0x210 */ { LDR R1, R0, #1 },
        /* 0x211 */ { ADD R1, R1, #10 },
        /* 0x212 */ { STR R1, R0, #1 },

        // Load ISR Remaining:
        /* 0x213 */ { LDR R1, R0, #0 },

        // If we've hit 0, read KBDR and exit.
        /* 0x214 */ { BRnp #3 },
        /* 0x215 */ { LDR R1, R0, #2 },
        /* 0x216 */ { LDR R1, R1, #0 },
        /* 0x217 */ { RTI },

        // Otherwise, decrement, store, and exit.
        /* 0x218 */ { ADD R1, R1, #-1 },
        /* 0x219 */ { STR R1, R0, #0 },
        /* 0x21A */ { RTI },
    ],
    // Some arbitrarily high number that's definitely long enough for the ISR to
    // run 6 times.
    steps: 50_000,
    memory: {
        // If this (ISR Remaining) is not 0, the ISR did not run multiple
        // times.
        // 0x220: 0,
        // If this (ISR Count) is not 6, the ISR did not stop running.
        0x223: 0,
        0x222: 0xFE02,
        0x220: 5,
        0x221: 6,
    },
    with io peripherals: { source as inp, sink as _out },
    pre: |_p| {
        inp.push('a');
    },
    post: |p| {
        // Interrupts enabled, ready bit low.
        let (kbsr_actual, kbsr_expected) = (
            p.get_word_unchecked(KBSR_ADDR),
            0b0100_0000_0000_0000,
        );
        eq!(kbsr_actual, kbsr_expected, "{kbsr_actual:#018b} vs. {kbsr_expected:#018b}");

    },
}

/*
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; init code
.orig x800
; Put our ISR in the interrupt vector table:
    ; compute the address corresponding to the keyboard
    ; interrupt's vector table entry, store into r0:
    LD  r0, INT_VEC_TABLE_BASE
    LD  r1, KBD_INTERRUPT_VEC_NUM
    ADD r0, r0, r1
    ; get the address of our keyboard isr:
    LD  r1, KBD_ISR_ADDR
    ; store:
    STR r1, r0, #0
; Enable interrupts (bit 14 of KBSR):
    LD  r0, KBSR
    LDR r1, r0, #0
    ; mask out bit 14:
    LD  r2, BIT_14_MASK
    AND r1, r1, r2
    ; set bit 14
    NOT r2, r2
    ADD r1, r1, r2
    ; store:
    STR r1, r0, #0
; Jump to user code:
    ADD r6, r6, #0
    BRz SET_UP_SSP
CONTINUE
    LD  r0, STARTING_PSR
    ADD r6, r6, #-1
    STR r0, r6, #0
    LD  r0, STARTING_PC
    ADD r6, r6, #-1
    STR r0, r6, #0
    RTI
SET_UP_SSP
    LD r6, STARTING_SSP
    BRnzp CONTINUE
STARTING_SSP
    .FILL x3000
INT_VEC_TABLE_BASE
    .FILL x0180
KBD_INTERRUPT_VEC_NUM
    .FILL x0
KBSR
    .FILL xFE00
KBDR
    .FILL xFE02
BIT_14_MASK
    .FILL xBFFF
STARTING_PC
    .FILL x3000
KBD_ISR_ADDR
    .FILL x4000
STARTING_PSR
    .FILL x8002
.end
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; user program
.orig x3000
USER_PROG
    LEA r0, MSG_START
    PUTS
; Spin, endlessly:
LOOP
    BRnzp LOOP
MSG_START
    .STRINGZ "hello!\n"
.end
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; ISR
.orig x4000
KBD_ISR
    LEA r0, MSG
    PUTS
    RTI
MSG
    .STRINGZ "in the keyboard ISR!\n"
.end
*/
