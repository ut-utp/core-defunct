//! Home of the workhorse of this crate: `interp_test_runner`; the thing that
//! actually runs the interpreter.

use lc3_isa::{Addr, Instruction, Word, USER_PROGRAM_START_ADDR};
use lc3_traits::memory::Memory;
use lc3_traits::peripherals::{Peripherals, PeripheralsWrapper};
use lc3_baseline_sim::interp::{
    InstructionInterpreter, Interpreter, InterpreterBuilder, MachineState
};
use core::convert::TryInto;

use pretty_assertions::assert_eq;

// TODO: add a trace option..

#[inline]
pub fn interp_test_runner<M: Memory + Default + Clone, P: Default + Peripherals, PF, TF>
(
    prefilled_memory_locations: Vec<(Addr, Word)>,
    insn_offset_addr: Word,
    insns: Vec<Instruction>,
    starting_pc: Word,
    num_steps: Option<usize>,
    regs: [Option<Word>; 8],
    pc: Option<Addr>,
    memory_locations: Vec<(Addr, Word)>,
    setup_func: PF,
    teardown_func: TF,
    alt_memory: Option<(M, Addr)>,
    alt_peripherals: Option<P>,
)
where
    for<'p> PF: FnOnce(&'p mut PeripheralsWrapper<P>),
    for<'i> TF: FnOnce(&'i mut Interpreter<M, P>), // Note: we could pass by value
                                                   // since this is the last thing
                                                   // we do.
{
    let mut addr = insn_offset_addr;

    let interp_builder = InterpreterBuilder::new().with_defaults();

    let interp_builder = if let Some(peripherals) = alt_peripherals {
        interp_builder.with_peripherals(peripherals)
    } else {
        interp_builder
    };

    let mut interp: Interpreter<M, P> = if let Some((mem, addr)) = alt_memory {
        let mut int: Interpreter<M, P> = interp_builder
            .with_memory(mem)
            .build();

        int.reset();
        int.set_pc(starting_pc);

        int
    } else {
        let mut int = interp_builder.build();

        int.reset();
        int.set_pc(starting_pc);

        int
    };

    // Run the setup func:
    setup_func(&mut *interp);

    // Prefill the memory locations:
    for (addr, word) in prefilled_memory_locations.iter() {
        // Crashes on ACVs! (they should not happen at this point)
        interp.set_word(*addr, *word).unwrap()
    }

    for insn in insns {
        // let enc = Into::<u16>::into(insn);
        // println!("{:?}", insn);
        // println!("{:#04X} -> {:?}", enc, Instruction::try_from(enc));
        interp.set_word_unchecked(addr, insn.into());
        // println!(
        //     "{:?}",
        //     Instruction::try_from(interp.get_word_unchecked(addr))
        // );

        addr += 1;
    }

    if let Some(num_steps) = num_steps {
        for _ in 0..num_steps {
            // println!("step: x{0:4X}", interp.get_pc());
            let _ = interp.step();
        }
    } else {
        while let MachineState::Running = interp.step() { }
    }

    // Check PC:
    if let Some(expected_pc) = pc {
        let actual_pc = interp.get_pc();
        assert_eq!(
            expected_pc,
            actual_pc,
            "Expected PC = {:#04X}, got {:#04X}",
            expected_pc,
            actual_pc
        );
    }


    // Check registers:
    for (idx, r) in regs.iter().enumerate() {
        if let Some(reg_word) = r {
            let val = interp.get_register((idx as u8).try_into().unwrap());
            assert_eq!(
                *reg_word,
                val,
                "Expected R{} to be {:?}, was {:?}",
                idx,
                *reg_word,
                val,
            );
        }
    }

    // Check memory:
    for (addr, word) in memory_locations.iter() {
        let val = interp.get_word_unchecked(*addr);
        assert_eq!(
            *word, val,
            "Expected memory location {:#04X} to be {:#04X}, was {:#04X}",
            addr, *word, val
        );
    }

    // Run the teardown func:
    teardown_func(&mut interp);
}
