
use lc3_baseline_sim::*;
use lc3_isa::{insn, Instruction, Reg, Addr, Word};
use lc3_traits::memory::Memory;
use lc3_traits::peripherals::Peripherals;

use lc3_baseline_sim::interp::{Interpreter, InstructionInterpreter, MachineState};

use lc3_shims::peripherals::PeripheralsShim;
use lc3_shims::memory::MemoryShim;

#[cfg(test)]
mod tests {
    use super::*;

    use Reg::*;

    use std::convert::TryInto;

    // Test that the instructions work
    // Test that the unimplemented instructions do <something>

    fn interp_test_runner<'a, M: Memory + Default, P: Peripherals<'a>>(
        insns: Vec<Instruction>,
        num_steps: Option<usize>,
        regs: [Option<Word>; 8],
        pc: Addr,
        memory_locations: Vec<(Addr, Word)>,
    )
    // where for<'p> P: Peripherals<'p>
    {
        let mut interp = Interpreter::<M, P>::default();

        let mut addr = 0x3000;
        interp.reset();
        interp.set_pc(addr);

        for insn in insns {
            interp.set_word_unchecked(addr, insn.into());
            addr += 1;
        }

        if let Some(num_steps) = num_steps {
            for _ in 0..num_steps {
                // println!("step: x{0:4X}", interp.get_pc());
                interp.step();
            }
        } else {
            while let MachineState::Running = interp.get_machine_state() {
                interp.step();
            }
        }

        // Check PC:
        assert_eq!(pc, interp.get_pc());

        // Check registers:
        for (idx, r) in regs.iter().enumerate() {
            if let Some(reg_word) = r {
                assert_eq!(interp.get_register((idx as u8).try_into().unwrap()), *reg_word);
            }
        }

        // Check memory:
        for (addr, word) in memory_locations.iter() {
            assert_eq!(interp.get_word_unchecked(*addr), *word);
        }
    }

    macro_rules! single {
        ($name:ident, insn: {$($insn:tt)*} steps: $steps:expr, ending_pc: $pc:literal, regs: { $($r:literal: $v:literal),* }, memory: { $($addr:literal: $val:literal),* }) => {
        #[test]
        fn $name() {

            #[allow(unused_mut)]
            let mut regs: [Option<Word>; Reg::NUM_REGS] = [None, None, None, None, None, None, None, None];
            $(regs[Into::<u8>::into($r) as usize] = Some($v);)*

            #[allow(unused_mut)]
            let mut checks: Vec<(Addr, Word)> = Vec::new();
            $(checks.push(($addr, $val));)*

            interp_test_runner::<MemoryShim, PeripheralsShim>(
                vec![insn!($($insn)*)],
                $steps,
                regs,
                $pc,
                checks
            );
        }};
    }

    single! {
        no_op,
        insn: { BRnzp #-1 }
        steps: Some(1),
        ending_pc: 0x3000,
        regs: {},
        memory: {}
    }

    single! {
        branch_simple,
        insn: { BRz #3 }
        steps: Some(1),
        ending_pc: 0x3004,
        regs: {},
        memory: {}
    }

    // #[test]
    // //NO-OP do nothing run a cycle test
    // fn no_op() {
    //     // single! {
    //     //     insn: { BRnzp #-1 }
    //     //     steps: Some(1),
    //     //     ending_pc: 0x3000,
    //     //     regs: {},
    //     //     memory: {}
    //     // }
    //     // interp_test_runner::<MemoryShim, PeripheralsShim<'a>>(
    //     //     vec![insn!(BRnzp #-1)],
    //     //     Some(1),
    //     //     [None, None, None, None, None, None, None, None],
    //     //     0x3000,
    //     //     vec![],
    //     // )
    // }

    #[should_panic]
    single! {
        no_op_fail,
        insn: { BRnzp #2 }
        steps: Some(1),
        ending_pc: 0x3000,
        regs: {},
        memory: {}
    }

    // #[test]
    // #[should_panic]
    // fn no_op_fail() {
    //     single! {
    //         insn: { BRnzp #2 }
    //         steps: Some(1),
    //         ending_pc: 0x3000,
    //         regs: {},
    //         memory: {}
    //     }
    //     // interp_test_runner::<MemoryShim, PeripheralsShim<'a>>(
    //     //     // vec![Instruction::Br {
    //     //     //     n: true,
    //     //     //     z: true,
    //     //     //     p: true,
    //     //     //     offset9: 67,
    //     //     // }],
    //     //     vec![insn!(BRnzp #67)],
    //     //     Some(1),
    //     //     [None, None, None, None, None, None, None, None],
    //     //     0x3000,
    //     //     vec![],
    //     // )
    // }
    // //0+1=1 Basic Add
    // #[test]
    // fn add_reg_test() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![
    //             Instruction::AddImm {
    //                 dr: R1,
    //                 sr1: R1,
    //                 imm5: 1,
    //             },
    //             AddReg {
    //                 dr: 2,
    //                 sr1: 1,
    //                 sr2: 0,
    //             },
    //         ],
    //         Some(1),
    //         [Some(0), Some(1), Some(1), None, None, None, None, None],
    //         0x3001,
    //         vec![],
    //     )
    // }
    // //AddImm Test with R0(0) + !
    // #[test]
    // fn AddImmTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![Instruction::AddImm {
    //             dr: R0,
    //             sr1: R0,
    //             imm5: 1,
    //         }],
    //         Some(1),
    //         [1, None, None, None, None, None, None, None],
    //         0x3001,
    //         vec![],
    //     )
    // }
    // //AndReg Test with R0(1) and R1(2) to R0(expected 3)
    // #[test]
    // fn AndRegTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![
    //             Instruction::AddImm {
    //                 dr: R0,
    //                 sr1: R0,
    //                 imm5: 1,
    //             },
    //             AddImm {
    //                 dr: R1,
    //                 sr1: R1,
    //                 imm5: 2,
    //             },
    //             AndReg {
    //                 dr: R0,
    //                 sr1: R0,
    //                 sr2: R1,
    //             },
    //         ],
    //         Some(3),
    //         [3, 2, None, None, None, None, None, None],
    //         0x3003,
    //         vec![],
    //     )
    // }
    // //AndImm Test with R1 (1) and 0
    // #[test]
    // fn AndImmTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![
    //             Instruction::AddImm {
    //                 dr: R1,
    //                 sr1: R1,
    //                 imm5: 1,
    //             },
    //             AndImm {
    //                 dr: R1,
    //                 sr1: R1,
    //                 imm5: 0,
    //             },
    //         ],
    //         Some(2),
    //         [0, None, None, None, None, None, None, None],
    //         0x3002,
    //         vec![],
    //     )
    // }
    // //ST Test which stores 1 into x3001
    // #[test]
    // fn StTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![
    //             Instruction::AddImm {
    //                 dr: R0,
    //                 sr1: R0,
    //                 imm5: 1,
    //             },
    //             St { sr: R0, offset9: 0 },
    //         ],
    //         Some(2),
    //         [1, None, None, None, None, None, None, None],
    //         0x3002,
    //         vec![(0x3001, 1)],
    //     )
    // }
    // //LD Test with R0 and memory
    // #[test]
    // fn LdTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![
    //             Instruction::AddImm {
    //                 dr: R0,
    //                 sr1: R0,
    //                 imm5: 1,
    //             },
    //             St { sr: R0, offset9: 1 },
    //             Ld { dr: R0, offset9: 0 },
    //         ],
    //         Some(3),
    //         [3001, None, None, None, None, None, None, None],
    //         0x3003,
    //         vec![(0x3001, 1)],
    //     )
    // }
    // //LDR Test with R0 and memory
    // #[test]
    // fn LdrTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![
    //             Instruction::AddImm {
    //                 dr: R0,
    //                 sr1: R0,
    //                 imm5: 1,
    //             },
    //             St { sr: R0, offset9: 0 },
    //             Ldr {
    //                 dr: R1,
    //                 offset9: -1,
    //             },
    //         ],
    //         Some(3),
    //         [1, 3001, None, None, None, None, None, None],
    //         0x3003,
    //         vec![(0x3001, 1)],
    //     )
    // }
    // //Load x3000 into R1
    // #[test]
    // fn LeaTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![Instruction::Lea { dr: R0, offset9: 0 }],
    //         Some(1),
    //         [3000, None, None, None, None, None, None, None],
    //         0x3001,
    //         vec![],
    //     )
    // }
    // // STR test with offset store into lea using 3000
    // #[test]
    // fn StrTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![
    //             Instruction::Lea { dr: R1, offset9: 0 },
    //             Lea { dr: R2, offset9: 1 },
    //             Str {
    //                 sr: R2,
    //                 base: R1,
    //                 offset6: 1,
    //             },
    //         ],
    //         Some(3),
    //         [None, None, None, None, None, None, None, None],
    //         0x3003,
    //         vec![(x3004, 3000)],
    //     )
    // }
    // //not test
    // #[test]
    // fn NotTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![
    //             Instruction::AddImm {
    //                 dr: R0,
    //                 sr1: R0,
    //                 imm5: 1,
    //             },
    //             Not { dr: R1, sr: R0 },
    //         ],
    //         Some(2),
    //         [1, 0, None, None, None, None, None, None],
    //         0x3002,
    //         vec![],
    //     )
    // }
    // //ldi Test using location 3000 and loading value of memory into register, using 3002 and 3001 holding 3000 as reference
    // #[test]
    // fn LdiTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![
    //             Instruction::Lea { dr: R0, offset9: 0 },
    //             St { sr: R0, offset9: 0 },
    //             St {
    //                 sr: R0,
    //                 offset9: -2,
    //             },
    //             Ldi {
    //                 dr: R2,
    //                 offset9: -1,
    //             },
    //         ],
    //         Some(4),
    //         [1, None, 3000, None, None, None, None, None],
    //         0x3004,
    //         vec![(x3001, 3000), (x3000, 3000)],
    //     )
    // }
    // //jumps to R7 register, loaded with memory address 3005
    // #[test]
    // fn RetTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![Instruction::Lea { dr: R7, offset9: 5 }, Ret],
    //         Some(2),
    //         [None, None, None, None, None, None, None, 3005],
    //         0x3005,
    //         vec![],
    //     )
    // }
    // //STI test, stores 3000 in register 1 and sets that to the memory at x3002 so sti writes to memory location 3000
    // #[test]
    // fn StiTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![
    //             Instruction::Lea { dr: R0, offset9: 0 },
    //             St { sr: R0, offset6: 2 },
    //             AddImm {
    //                 dr: R3,
    //                 sr1: R3,
    //                 imm5: 1,
    //             },
    //             Sti { sr: R3, offset9: 0 },
    //         ],
    //         Some(4),
    //         [3000, None, None, 1, None, None, None, None],
    //         0x3004,
    //         vec![(x3003, 3000), (x3000, 1)],
    //     )
    // }
    // //Jump Test, switch PC to value in register
    // #[test]
    // fn JmpTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![Instruction::Lea { dr: R0, offset9: 0 }, Jmp { base: R0 }],
    //         Some(2),
    //         [3000, None, None, None, None, None, None, None],
    //         0x3000,
    //         vec![],
    //     )
    // }
    // //jsrr test, jumps to location 3005 and stores 3001 in r7
    // #[test]
    // fn JsrrTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![Instruction::Lea { dr: R0, offset9: 5 }, Jsrr { base: R0 }],
    //         Some(2),
    //         [3000, None, None, None, None, None, None, 3001],
    //         0x3005,
    //         vec![],
    //     )
    // }
    // //jsr test, jumps back to queue location from r7
    // #[test]
    // fn JsrTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![
    //             Instruction::Lea { dr: R0, offset9: 5 },
    //             St { sr: R0, offset6: 2 },
    //             Jsr { offset11: 1 },
    //         ],
    //         Some(3),
    //         [3000, None, None, None, None, None, None, 3001],
    //         0x3000,
    //         vec![],
    //     )
    // }
}