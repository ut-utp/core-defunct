//! TODO!
use super::mem_mapped::{MemMapped, MemMappedSpecial /*, BSP, PSR*/, MCR};

use lc3_isa::{
    Addr, Instruction,
    Reg::{self, *},
    Word, ACCESS_CONTROL_VIOLATION_EXCEPTION_VECTOR, ILLEGAL_OPCODE_EXCEPTION_VECTOR,
    INTERRUPT_VECTOR_TABLE_START_ADDR, MEM_MAPPED_START_ADDR,
    PRIVILEGE_MODE_VIOLATION_EXCEPTION_VECTOR, TRAP_VECTOR_TABLE_START_ADDR,
    USER_PROGRAM_START_ADDR,
};
use lc3_traits::{control::metadata::{Identifier, ProgramMetadata, Version, version_from_crate}, peripherals::{PeripheralsWrapper, PeripheralsExt}};
use lc3_traits::control::load::{PageIndex, PAGE_SIZE_IN_WORDS};
use lc3_traits::control::control::MAX_CALL_STACK_DEPTH;
use lc3_traits::peripherals::{gpio::GpioPinArr, timers::TimerArr};
use lc3_traits::{memory::Memory, peripherals::Peripherals};
use lc3_traits::peripherals::{gpio::Gpio, input::Input, output::Output, timers::Timers};
use lc3_traits::error::Error;
use crate::mem_mapped::Interrupt;

use core::any::TypeId;
use core::convert::TryInto;
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};
use core::sync::atomic::AtomicBool;
use core::ops::{Deref, DerefMut};
use core::cell::Cell;

// TODO: Break up this file!

// use the usual trick to bound an associated type without having the bound link
// into users of the trait
//
// See: https://stackoverflow.com/a/69386814
// An example: https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=709222486f15d4f46290d387b9d92652

pub trait DerefsIntoPeripherals: DerefMut + Deref<Target = Self::P> {
    type P: Peripherals + Sized;
}

impl<P: Deref + DerefMut> DerefsIntoPeripherals for P
where
    <P as Deref>::Target: Peripherals + Sized,
{
    type P = <P as Deref>::Target;
}

// TODO: name?
pub trait InstructionInterpreterPeripheralAccess:
    InstructionInterpreter + DerefsIntoPeripherals
    // TODO: revisit...
where {
    fn get_peripherals(&self) -> &<Self as Deref>::Target {
        self.deref()
    }

    fn get_peripherals_mut(&mut self) -> &mut <Self as Deref>::Target {
        self.deref_mut()
    }

    // TODO: explain
    //
    // this gives you a type that, for convenience, has proxied impls of all the
    // peripheral traits on it.
    //
    // requires `&mut` though which is why we don't use it in the interpreter's
    // source code
    fn peripherals_wrapper(&mut self) -> PeripheralsWrapper<'_, Self::Target> {
        PeripheralsExt::get_peripherals_wrapper(self)
    }

    fn peri(&mut self) -> PeripheralsWrapper<'_, Self::Target> {
        self.peripherals_wrapper()
    }

    fn get_device_reg<M: MemMapped>(&self) -> Result<M, Acv> {
        M::from(self)
    }

    fn set_device_reg<M: MemMapped>(&mut self, value: Word) -> WriteAttempt {
        M::set(self, value)
    }

    fn update_device_reg<M: MemMapped>(&mut self, func: impl FnOnce(M) -> Word) -> WriteAttempt {
        M::update(self, func)
    }

    fn get_special_reg<M: MemMappedSpecial>(&self) -> M {
        M::from_special(self)
    }

    fn set_special_reg<M: MemMappedSpecial>(&mut self, value: Word) {
        M::set_special(self, value)
    }

    fn update_special_reg<M: MemMappedSpecial>(&mut self, func: impl FnOnce(M) -> Word) {
        M::update(self, func).unwrap()
    }

    fn reset_peripherals(&mut self) {
        use lc3_traits::peripherals::gpio::{GPIO_PINS, GpioState};
        use lc3_traits::peripherals::adc::{Adc, ADC_PINS, AdcState};
        use lc3_traits::peripherals::pwm::{Pwm, PWM_PINS, PwmState};
        use lc3_traits::peripherals::timers::{TIMERS, TimerMode, TimerState};
        use lc3_traits::peripherals::clock::Clock;

        // TODO: do something with errors here?
        let p = self.get_peripherals_mut();

        let gpio = p.get_gpio_mut();
        for pin in GPIO_PINS.iter() {
            let _ = gpio.set_state(*pin, GpioState::Disabled);
            gpio.reset_interrupt_flag(*pin);
        }

        let adc = p.get_adc_mut();
        for pin in ADC_PINS.iter() {
            let _ = adc.set_state(*pin, AdcState::Disabled);
        }

        let pwm = p.get_pwm_mut();
        for pin in PWM_PINS.iter() {
            pwm.set_state(*pin, PwmState::Disabled);
            pwm.set_duty_cycle(*pin, 0);
        }

        let timers = p.get_timers_mut();
        for id in TIMERS.iter() {
            timers.set_mode(*id, TimerMode::SingleShot);
            timers.set_state(*id, TimerState::Disabled);
            timers.reset_interrupt_flag(*id);
        }

        Clock::set_milliseconds(p.get_clock_mut(), 0);
        Input::reset_interrupt_flag(p.get_input_mut());
        Output::reset_interrupt_flag(p.get_output_mut());
    }
}

pub trait InstructionInterpreter:
    Index<Reg, Output = Word> + IndexMut<Reg, Output = Word> + Sized
{
    const ID: Identifier = Identifier::new_from_str_that_crashes_on_invalid_inputs("Insn");
    const VER: Version = Version::empty()
        .pre_from_str_that_crashes_on_invalid_inputs("????");

    fn step(&mut self) -> MachineState;

    fn set_pc(&mut self, addr: Addr);
    fn get_pc(&self) -> Addr;

    // Checked access:
    fn set_word(&mut self, addr: Addr, word: Word) -> WriteAttempt;
    fn get_word(&self, addr: Addr) -> ReadAttempt;

    fn set_word_unchecked(&mut self, addr: Addr, word: Word);
    fn get_word_unchecked(&self, addr: Addr) -> Word;

    fn set_word_force_memory_backed(&mut self, addr: Addr, word: Word);
    fn get_word_force_memory_backed(&self, addr: Addr) -> Word;

    fn get_register(&self, reg: Reg) -> Word { self[reg] }
    fn set_register(&mut self, reg: Reg, word: Word) { self[reg] = word; }

    fn get_machine_state(&self) -> MachineState;
    fn reset(&mut self);
    fn halt(&mut self); // TODO: have the MCR set this, etc.

    fn set_error(&self, err: Error);
    fn get_error(&self) -> Option<Error>;

    fn get_call_stack(&self) -> [Option<(Addr, ProcessorMode)>; MAX_CALL_STACK_DEPTH];
    fn get_call_stack_depth(&self) -> u64;

    // Taken straight from Memory:
    fn commit_page(&mut self, page_idx: PageIndex, page: &[Word; PAGE_SIZE_IN_WORDS as usize]);

    fn get_program_metadata(&self) -> ProgramMetadata;
    fn set_program_metadata(&mut self, metadata: ProgramMetadata);

    // Until TypeId::of is a const function, this can't be an associated const:
    fn type_id() -> TypeId { core::any::TypeId::of::<Instruction>() }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Acv;

pub type ReadAttempt = Result<Word, Acv>;

pub type WriteAttempt = Result<(), Acv>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MachineState {
    Running,
    Halted,
}

impl MachineState {
    const fn new() -> Self {
        MachineState::Halted
    }
}

impl Default for MachineState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct PeripheralInterruptFlags {
    pub gpio: GpioPinArr<AtomicBool>, // No payload; just tell us if a rising edge has happened
    // adc: AdcPinArr<bool>, // We're not going to have Adc Interrupts
    // pwm: PwmPinArr<bool>, // No Pwm Interrupts
    pub timers: TimerArr<AtomicBool>, // No payload; timers don't actually expose counts anyways
    // clock: bool, // No Clock Interrupt
    pub input: AtomicBool, // No payload; check KBDR for the current character
    pub output: AtomicBool, // Technically this has an interrupt, but I have no idea why; UPDATE: it interrupts when it's ready to accept more data
                        // display: bool, // Unless we're exposing vsync/hsync or something, this doesn't need an interrupt
}

// TODO: move ^ to device support?

impl PeripheralInterruptFlags {
    pub const fn new() -> Self {
        macro_rules! b {
            () => {
                AtomicBool::new(false)
            };
        }

        // TODO: make this less gross..
        Self {
            gpio: GpioPinArr([b!(), b!(), b!(), b!(), b!(), b!(), b!(), b!()]),
            timers: TimerArr([b!(), b!()]),
            input: AtomicBool::new(false),
            output: AtomicBool::new(false),
        }
    }
}

impl Default for PeripheralInterruptFlags {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct CallStack {
    stack: [Option<(Addr, ProcessorMode)>; MAX_CALL_STACK_DEPTH],
    depth: u64,
}

impl CallStack {
    pub const fn new() -> Self {
        Self {
            stack: [None; MAX_CALL_STACK_DEPTH],
            depth: 0,
        }
    }

    // Push subroutine address to call stack if stack is not full
    // Always increments depth
    // -> true if pushed, false otherwise
    pub fn push(&mut self, subroutine: Addr, in_user_mode: bool) -> bool {
        let mut success = false;

        // Check if stack is not full
        if self.depth < MAX_CALL_STACK_DEPTH as u64 {
            let processor_mode = if in_user_mode {ProcessorMode::User} else {ProcessorMode::Supervisor};
            self.stack[self.depth as usize] = Some((subroutine, processor_mode));
            success = true;
        }

        // Increment depth
        self.depth = match self.depth.checked_add(1) {
            Some(val) => val,
            None => panic!("Overflowed depth of call stack!"),
        };

        success
    }

    // Pop subroutine address off of call stack if top of stack matches current depth
    // Always decrements depth
    // -> true if popped, false otherwise
    pub fn pop(&mut self) -> bool {
        let mut success = false;

        // Decrement depth, saturates at 0 (unsigned)
        self.depth = self.depth.saturating_sub(1);

        // Check if depth exceeds max saved addrs
        if self.depth < MAX_CALL_STACK_DEPTH as u64 {
            self.stack[self.depth as usize] = None;
            success = true;
        }

        success
    }
}

// #[derive(Debug, Default, Clone)] // TODO: Clone
#[derive(Debug)]
pub struct Interpreter<M: Memory, P: Peripherals> {
    memory: M,
    peripherals: P,
    regs: [Word; Reg::NUM_REGS],
    pc: Word, //TODO: what should the default for this be
    state: MachineState,
    error: Cell<Option<Error>>,
    call_stack: CallStack,
}

impl<M: Memory + Default, P: Peripherals + Default> Default for Interpreter<M, P> {
    fn default() -> Self {
        InterpreterBuilder::new()
            .with_defaults()
            .build()
    }
}

// TODO: replace with a builder macro...

#[derive(Debug)]
pub struct Set;
#[derive(Debug)]
pub struct NotSet;

#[derive(Debug)]
struct InterpreterBuilderData<M: Memory, P>
where
    P: Peripherals,
{
    memory: Option<M>,
    peripherals: Option<P>,
    regs: Option<[Word; Reg::NUM_REGS]>,
    pc: Option<Word>,
    state: Option<MachineState>,
}

#[derive(Debug)]
pub struct InterpreterBuilder<
    M: Memory,
    P,
    Mem = NotSet,
    Perip = NotSet,
    Regs = NotSet,
    Pc = NotSet,
    State = NotSet,
> where
    P: Peripherals,
{
    data: InterpreterBuilderData<M, P>,
    _mem: PhantomData<Mem>,
    _perip: PhantomData<Perip>,
    _regs: PhantomData<Regs>,
    _pc: PhantomData<Pc>,
    _state: PhantomData<State>,
}

impl<M: Memory, P, Mem, Perip, Regs, Pc, State>
    InterpreterBuilder<M, P, Mem, Perip, Regs, Pc, State>
where
    P: Peripherals,
{
    fn with_data(data: InterpreterBuilderData<M, P>) -> Self {
        Self {
            data,
            _mem: PhantomData,
            _perip: PhantomData,
            _regs: PhantomData,
            _pc: PhantomData,
            _state: PhantomData,
        }
    }
}

impl<M: Memory, P> InterpreterBuilder<M, P, NotSet, NotSet, NotSet, NotSet, NotSet>
where
    P: Peripherals,
{
    pub fn new() -> Self {
        Self {
            data: InterpreterBuilderData {
                memory: None,
                peripherals: None,
                regs: None,
                pc: None,
                state: None,
            },
            _mem: PhantomData,
            _perip: PhantomData,
            _regs: PhantomData,
            _pc: PhantomData,
            _state: PhantomData,
        }
    }
}

impl<M: Memory, P, Mem, Perip, Regs, Pc, State>
    InterpreterBuilder<M, P, Mem, Perip, Regs, Pc, State>
where
    P: Peripherals,
    P: Default,
    M: Default,
{
    pub fn with_defaults(self) -> InterpreterBuilder<M, P, Set, Set, Set, Set, Set> {
        InterpreterBuilder::with_data(InterpreterBuilderData {
            memory: Some(Default::default()),
            peripherals: Some(Default::default()),
            regs: Some(Default::default()),
            pc: Some(Default::default()),
            state: Some(Default::default()),
        })
    }
}

impl<M: Memory, P, Mem, Perip, Regs, Pc, State>
    InterpreterBuilder<M, P, Mem, Perip, Regs, Pc, State>
where
    P: Peripherals,
{
    pub fn with_memory(
        self,
        memory: M,
    ) -> InterpreterBuilder<M, P, Set, Perip, Regs, Pc, State> {
        InterpreterBuilder::with_data(InterpreterBuilderData {
            memory: Some(memory),
            ..self.data
        })
    }
}

impl<M: Memory + Default, P, Mem, Perip, Regs, Pc, State>
    InterpreterBuilder<M, P, Mem, Perip, Regs, Pc, State>
where
    P: Peripherals,
{
    pub fn with_default_memory(
        self,
    ) -> InterpreterBuilder<M, P, Set, Perip, Regs, Pc, State> {
        self.with_memory(Default::default())
    }
}

impl<M: Memory, P, Mem, Perip, Regs, Pc, State>
    InterpreterBuilder<M, P, Mem, Perip, Regs, Pc, State>
where
    P: Peripherals,
{
    pub fn with_peripherals(
        self,
        peripherals: P,
    ) -> InterpreterBuilder<M, P, Mem, Set, Regs, Pc, State> {
        InterpreterBuilder::with_data(InterpreterBuilderData {
            peripherals: Some(peripherals),
            ..self.data
        })
    }

    pub fn with_default_peripherals(
        self,
    ) -> InterpreterBuilder<M, P, Mem, Set, Regs, Pc, State> where P: Default {
        self.with_peripherals(Default::default())
    }
}


// TODO: do we want to allow people to set the starting register values?
impl<M: Memory, P, Mem, Perip, Regs, Pc, State>
    InterpreterBuilder<M, P, Mem, Perip, Regs, Pc, State>
where
    P: Peripherals,
{
    pub fn with_regs(
        self,
        regs: [Word; Reg::NUM_REGS],
    ) -> InterpreterBuilder<M, P, Mem, Perip, Set, Pc, State> {
        InterpreterBuilder::with_data(InterpreterBuilderData {
            regs: Some(regs),
            ..self.data
        })
    }

    pub fn with_default_regs(
        self,
    ) -> InterpreterBuilder<M, P, Mem, Perip, Set, Pc, State> {
        self.with_regs(Default::default())
    }
}

// TODO: do we want to allow people to set the starting pc?
impl<M: Memory, P, Mem, Perip, Regs, Pc, State>
    InterpreterBuilder<M, P, Mem, Perip, Regs, Pc, State>
where
    P: Peripherals,
{
    pub fn with_pc(
        self,
        pc: Word,
    ) -> InterpreterBuilder<M, P, Mem, Perip, Regs, Set, State> {
        InterpreterBuilder::with_data(InterpreterBuilderData {
            pc: Some(pc),
            ..self.data
        })
    }

    pub fn with_default_pc(
        self,
    ) -> InterpreterBuilder<M, P, Mem, Perip, Regs, Set, State> {
        self.with_pc(Default::default())
    }
}

// TODO: do we want to allow people to set the starting machine state?
impl<M: Memory, P, Mem, Perip, Regs, Pc, State>
    InterpreterBuilder<M, P, Mem, Perip, Regs, Pc, State>
where
    P: Peripherals,
{
    pub fn with_state(
        self,
        state: MachineState,
    ) -> InterpreterBuilder<M, P, Mem, Perip, Regs, Pc, Set> {
        InterpreterBuilder::with_data(InterpreterBuilderData {
            state: Some(state),
            ..self.data
        })
    }

    pub fn with_default_state(
        self,
    ) -> InterpreterBuilder<M, P, Mem, Perip, Regs, Pc, Set> {
        self.with_state(Default::default())
    }
}

impl<M: Memory, P> InterpreterBuilder<M, P, Set, Set, Set, Set, Set>
where
    P: Peripherals,
{
    pub fn build(self) -> Interpreter<M, P> {
        Interpreter::new(
            self.data.memory.unwrap(),
            self.data.peripherals.unwrap(),
            self.data.regs.unwrap(),
            self.data.pc.unwrap(),
            self.data.state.unwrap(),
        )
    }
}

impl<M: Memory, P: Peripherals> Interpreter<M, P> {
    pub fn new(
        memory: M,
        peripherals: P,
        regs: [Word; Reg::NUM_REGS],
        pc: Word,
        state: MachineState,
    ) -> Self {
        let mut interp = Self {
            memory,
            peripherals,
            regs,
            pc,
            state,
            error: Cell::new(None),
            call_stack: CallStack::new(),
        };

        interp.reset(); // TODO: remove pc/regs options from the interpreter builder
        interp
    }
}

impl<M: Memory, P: Peripherals> Index<Reg> for Interpreter<M, P> {
    type Output = Word;

    fn index(&self, reg: Reg) -> &Self::Output {
        &self.regs[TryInto::<usize>::try_into(Into::<u8>::into(reg)).unwrap()]
    }
}

impl<M: Memory, P: Peripherals> IndexMut<Reg> for Interpreter<M, P> {
    fn index_mut(&mut self, reg: Reg) -> &mut Self::Output {
        &mut self.regs[TryInto::<usize>::try_into(Into::<u8>::into(reg)).unwrap()]
    }
}

impl<M: Memory, P: Peripherals> Deref for Interpreter<M, P> {
    type Target = P;

    fn deref(&self) -> &Self::Target {
        &self.peripherals
    }
}

impl<M: Memory, P: Peripherals> DerefMut for Interpreter<M, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.peripherals
    }
}

impl<M: Memory, P: Peripherals> InstructionInterpreterPeripheralAccess
    for Interpreter<M, P>
{ }

impl<M: Memory, P: Peripherals> Interpreter<M, P> {
    fn set_cc(&mut self, word: Word) {
        <PSR as MemMapped>::from(self).unwrap().set_cc(self, word)
    }

    fn get_cc(&self) -> (bool, bool, bool) {
        self.get_special_reg::<PSR>().get_cc()
    }

    fn push(&mut self, word: Word) -> WriteAttempt {
        // This function will *only ever push onto the system stack*:
        // TODO: report an error if it's in the I/O region somehow? it's not possible in regular use but it happened to me anyways (when getting the OS not to switch to user mode)
        if self[R6] <= lc3_isa::OS_START_ADDR {
            self.set_error(SystemStackOverflow);
            self.halt();
            return Err(Acv);    // TODO: Kind of an ACV, but not really?
        }

        self[R6] -= 1;
        self.set_word(self[R6], word)
    }

    // Take notice! This will not modify R6 if the read fails!
    // TODO: Is this correct?
    fn pop(&mut self) -> ReadAttempt {
        let word = self.get_word(self[R6])?;
        self[R6] += 1;

        Ok(word)
    }

    // TODO: Swap result out
    fn push_state(&mut self, saved_psr: Word) -> WriteAttempt {
        // Push the saved PSR and then the PC so that the PC gets popped first.
        // (Popping the PSR first could trigger an ACV)

        self.push(saved_psr)
            .and_then(|()| self.push(self.get_pc()))
    }

    fn restore_state(&mut self) -> Result<(), Acv> {
        // Update call stack on return
        self.pop_call_stack();

        // Restore the PC and then the PSR.
        self.pop()
            .map(|w| self.set_pc(w))
            .and_then(|()| self.pop().map(|w| self.set_special_reg::<PSR>(w)))
    }

    // Infallible since BSP is 'special'.
    fn swap_stacks(&mut self) {
        let (sp, bsp) = (self[R6], *self.get_special_reg::<BSP>());

        BSP::set_special(self, sp);
        self[R6] = bsp;
    }

    // TODO: execution event = exception, interrupt, or a trap
    // find a better name
    fn prep_for_execution_event(&mut self) {
        let mut psr = self.get_special_reg::<PSR>();

        // Need to save a temporary copy of the PSR before switching to supervisor mode
        let saved_psr: Word = *self.get_special_reg::<PSR>();

        // If we're in user mode..
        if psr.in_user_mode() {
            // ..switch to supervisor mode..
            psr.to_privileged_mode(self);

            // ..and switch the stacks:
            self.swap_stacks();
        } else {
            // We're assuming if PSR[15] is in supervisor mode, R6 is already
            // the supervisor stack pointer and BSR has the user stack pointer.
        }

        // We're in privileged mode now so this should only error if we've
        // overflowed our stack.
        if let Err(Acv) = self.push_state(saved_psr) {
            debug_assert_eq!(self.state, MachineState::Halted);
            return;
        }

//        self.get_special_reg::<PSR>().set_priority(self, 3);
    }

    fn handle_trap(&mut self, trap_vec: u8) {
        self.prep_for_execution_event();

        // Go to the trap routine:
        // (this should also not panic)
        self.pc = self
            .get_word(TRAP_VECTOR_TABLE_START_ADDR | (Into::<Word>::into(trap_vec)))
            .unwrap();

        self.push_call_stack(self.pc, self.get_special_reg::<PSR>().in_user_mode());
    }

    // TODO: find a word that generalizes exception and trap...
    // since that's what this handles
    fn handle_exception(&mut self, ex_vec: u8) {
        self.prep_for_execution_event();

        // Go to the exception routine:
        // (this should also not panic)
        self.pc = self
            .get_word(INTERRUPT_VECTOR_TABLE_START_ADDR | (Into::<Word>::into(ex_vec)))
            .unwrap();

        self.push_call_stack(self.pc, self.get_special_reg::<PSR>().in_user_mode());
    }

    fn handle_interrupt(&mut self, int_vec: u8, priority: u8) -> bool {
        // TODO: check that the ordering here is right

        // Make sure that the priority is high enough to interrupt:
        if self.get_special_reg::<PSR>().get_priority() > priority {
            // Gotta wait.
            return false;
        }

        // Haven't executed instruction at PC-1, so must store PC-1 on stack, not PC
        self.pc -= 1;

        self.handle_exception(int_vec);
        self.set_cc(0);
        self.get_special_reg::<PSR>().set_priority(self, priority);

        true
        // self.prep_for_execution_event();

        // // Go to the interrupt routine:
        // // (this should also not panic)
        // self.pc = self
        //     .get_word(INTERRUPT_VECTOR_TABLE_START_ADDR | (Into::<Word>::into(int_vec)))
        //     .unwrap();

        // true
    }

    fn check_interrupts(&mut self) -> bool {
        macro_rules! assert_in_priority_order {
            ($dev1: ty, $dev2: ty, $($rest:ty),*) => {
                sa::const_assert!(<$dev1>::PRIORITY >= <$dev2>::PRIORITY);

                assert_in_priority_order!($dev2, $($rest),*);
            };
            ($dev1: ty, $dev2: ty) => {
                sa::const_assert!(<$dev1>::PRIORITY >= <$dev2>::PRIORITY);
            }
        }

        macro_rules! int_devices {
            ($($dev:ty),* $(,)?) => {
                let cur_priority: u8 = self.get_special_reg::<PSR>().get_priority();
                $(
                    if <$dev>::PRIORITY <= cur_priority { return false; }
                    else if <$dev as Interrupt>::interrupt(self) {
                        <$dev as Interrupt>::reset_interrupt_flag(self);
                        return self.handle_interrupt(<$dev>::INT_VEC, <$dev>::PRIORITY);
                    }
                )*

                assert_in_priority_order!($($dev),*);
            }
        }

        int_devices!(
            KBSR, DSR, G0CR, G1CR, G2CR, G3CR, G4CR, G5CR, G6CR, G7CR, T0CR, T1CR
        );
        false
    }

    fn is_acv(&self, addr: Word) -> bool {
        // TODO: is `PSR::from_special(self).in_user_mode()` clearer?

        if self.get_special_reg::<PSR>().in_user_mode() {
            (addr < USER_PROGRAM_START_ADDR) | (addr >= MEM_MAPPED_START_ADDR)
        } else {
            false
        }
    }

    fn instruction_step_inner(&mut self, insn: Instruction) -> Result<(), Acv> {
        use Instruction::*;

        macro_rules! i {
            (PC <- $expr:expr) => {
                self.set_pc($expr);
            };
            (mem[$addr:expr] <- $expr:expr) => {
                self.set_word($addr, $expr)?;
            };
            ($dr:ident <- $expr:expr) => {
                self[$dr] = $expr;
                if insn.sets_condition_codes() {
                    self.set_cc(self[$dr]);
                }
            };
        }

        macro_rules! I {
            (PC <- $($rest:tt)*) => {{
                _insn_inner_gen!($);
                #[allow(unused_mut)]
                let mut pc: Addr;

                _insn_inner!(pc | $($rest)*);
                i!(PC <- pc);
            }};

            ([S+] PC <- $($rest:tt)*) => {{
                _insn_inner_gen!($);
                #[allow(unused_mut)]
                let mut pc: Addr;

                _insn_inner!(pc | $($rest)*);
                i!(PC <- pc);

                self.push_call_stack(pc, self.get_special_reg::<PSR>().in_user_mode());
            }};

            ([S-] PC <- $($rest:tt)*) => {{
                _insn_inner_gen!($);
                #[allow(unused_mut)]
                let mut pc: Addr;

                _insn_inner!(pc | $($rest)*);
                i!(PC <- pc);

                self.pop_call_stack();
            }};

            (mem[$($addr:tt)*] <- $($word:tt)*) => {{
                _insn_inner_gen!($);
                #[allow(unused_mut)]
                let mut addr: Addr;
                #[allow(unused_mut)]
                let mut word: Word;

                _insn_inner!(addr | $($addr)*);
                _insn_inner!(word | $($word)*);

                self.set_word(addr, word)?
            }};

            ($dr:ident <- $($rest:tt)*) => {{
                _insn_inner_gen!($);
                #[allow(unused_mut)]
                let mut word: Word;

                _insn_inner!(word | $($rest)*);

                i!($dr <- word);
            }};
        }

        macro_rules! _insn_inner_gen {
            ($d:tt) => {
                macro_rules! _insn_inner {
                    ($d nom:ident | R[$d reg:expr] $d ($d rest:tt)*) => { $d nom = self[$d reg]; _insn_inner!($d nom | $d ($d rest)*) };
                    ($d nom:ident | PC $d ($d rest:tt)*) => { $d nom = self.get_pc(); _insn_inner!($d nom | $d ($d rest)*) };
                    ($d nom:ident | mem[$d ($d addr:tt)*] $d ($d rest:tt)*) => {
                        $d nom = self.get_word({
                            let mut _addr_mem: Addr;
                            _insn_inner!(_addr_mem | $d ($d addr)*);
                            _addr_mem
                        }/* as Word*/)?;

                        _insn_inner!($d nom | $d ($d rest)*)
                    };
                    ($d nom:ident | + $d ($d rest:tt)*) => {
                        $d nom = $d nom.wrapping_add(
                        {
                            let mut _rhs_add;
                            _insn_inner!(_rhs_add | $d ($d rest)*);
                            _rhs_add
                        } as Word);
                    };
                    ($d nom:ident | & $d ($d rest:tt)*) => {
                        $d nom = $d nom & {
                            let mut _rhs_and;
                            _insn_inner!(_rhs_and | $d ($d rest)*);
                            _rhs_and
                        } as Word;
                    };
                    ($d nom:ident | ! $d ($d rest:tt)*) => {
                        $d nom = ! {
                            let mut _rhs_not: Word;
                            _insn_inner!(_rhs_not | $d ($d rest)*);
                            _rhs_not
                        } /*as Word*/;
                    };
                    ($d nom:ident | $d ident:ident $d ($d rest:tt)*) => {
                        $d nom = $d ident; _insn_inner!($d nom | $d ($d rest)*)
                    };
                    ($d nom:ident | ) => {};
                }
            }
        }

        match insn {
            AddReg { dr, sr1, sr2 } => I!(dr <- R[sr1] + R[sr2]),
            AddImm { dr, sr1, imm5 } => I!(dr <- R[sr1] + imm5),
            AndReg { dr, sr1, sr2 } => I!(dr <- R[sr1] & R[sr2]),
            AndImm { dr, sr1, imm5 } => I!(dr <- R[sr1] & imm5),
            Br { n, z, p, offset9 } => {
                let (cc_n, cc_z, cc_p) = self.get_cc();
                if n && cc_n || z && cc_z || p && cc_p {
                    I!(PC <- PC + offset9)
                }
            }
            Jmp { base: R7 } | Ret => I!([S-] PC <- R[R7]),
            Jmp { base } => I!(PC <- R[base]),
            Jsr { offset11 } => {
                I!(R7 <- PC);
                I!([S+] PC <- PC + offset11)
            }
            Jsrr { base } => {
                let (pc, new_pc) = (self.get_pc(), self[base]);
                I!([S+] PC <- new_pc);
                I!(R7 <- pc)
            }
            Ld { dr, offset9 } => I!(dr <- mem[PC + offset9]),
            Ldi { dr, offset9 } => I!(dr <- mem[mem[PC + offset9]]),
            Ldr { dr, base, offset6 } => I!(dr <- mem[R[base] + offset6]),
            Lea { dr, offset9 } => I!(dr <- PC + offset9),
            Not { dr, sr } => I!(dr <- !R[sr]),
            Rti => {
                if self.get_special_reg::<PSR>().in_privileged_mode() {
                    // This should never panic:
                    self.restore_state().unwrap();

                    // If we've gone back to user mode..
                    if self.get_special_reg::<PSR>().in_user_mode() {
                        // ..swap the stack pointers.
                        self.swap_stacks();
                    }
                } else {
                    // If RTI is called from user mode, raise the privilege mode
                    // exception:
                    self.handle_exception(PRIVILEGE_MODE_VIOLATION_EXCEPTION_VECTOR)
                }
            }
            St { sr, offset9 } => I!(mem[PC + offset9] <- R[sr]),
            Sti { sr, offset9 } => I!(mem[mem[PC + offset9]] <- R[sr]),
            Str { sr, base, offset6 } => I!(mem[R[base] + offset6] <- R[sr]),
            Trap { trapvec } => self.handle_trap(trapvec),
        }

        Ok(())
    }

    fn push_call_stack(&mut self, subroutine: Addr, in_user_mode: bool) -> bool {
        self.call_stack.push(subroutine, in_user_mode)
    }

    fn pop_call_stack(&mut self) -> bool {
        self.call_stack.pop()
    }
}

use super::mem_mapped::{
    KBSR, KBDR,
    DSR, DDR,
    BSP, PSR,
    G0CR, G0DR, G1CR, G1DR, G2CR, G2DR, G3CR, G3DR, G4CR, G4DR, G5CR, G5DR, G6CR, G6DR, G7CR, G7DR,
    A0CR, A0DR, A1CR, A1DR, A2CR, A2DR, A3CR, A3DR, A4CR, A4DR, A5CR, A5DR,
    P0CR, P0DR, P1CR, P1DR,
    CLKR,
    T0CR, T0DR, T1CR, T1DR
};
use lc3_traits::error::Error::SystemStackOverflow;
use lc3_traits::control::ProcessorMode;

impl<M: Memory, P: Peripherals> InstructionInterpreter for Interpreter<M, P> {
    const ID: Identifier = Identifier::new_from_str_that_crashes_on_invalid_inputs("Base");
    const VER: Version = version_from_crate!();

    fn step(&mut self) -> MachineState {
        if let state @ MachineState::Halted = self.get_machine_state() {
            return state;
        }

        // Increment PC (state 18):
        let current_pc = self.get_pc();
        self.set_pc(current_pc.wrapping_add(1)); // TODO: ???

        if self.check_interrupts() {
            return self.get_machine_state();
        };

        match self.get_word(current_pc).and_then(|w| match w.try_into() {
            Ok(insn) => self.instruction_step_inner(insn),
            Err(_) => {
                self.handle_exception(ILLEGAL_OPCODE_EXCEPTION_VECTOR);
                Ok(())
            }
        }) {
            Ok(()) => {}
            // Access control violation: triggered when getting the current instruction or when executing it
            Err(Acv) => self.handle_exception(ACCESS_CONTROL_VIOLATION_EXCEPTION_VECTOR),
        }

        self.get_machine_state()
    }

    fn set_pc(&mut self, addr: Addr) {
        self.pc = addr;
    }

    fn get_pc(&self) -> Addr {
        self.pc
    }

    // Checked access:
    fn set_word(&mut self, addr: Addr, word: Word) -> WriteAttempt {
        if self.is_acv(addr) {
            Err(Acv)
        } else {
            Ok(self.set_word_unchecked(addr, word))
        }
    }

    fn get_word(&self, addr: Addr) -> ReadAttempt {
        if self.is_acv(addr) {
            Err(Acv)
        } else {
            Ok(self.get_word_unchecked(addr))
        }
    }

    // Unchecked access:
    #[forbid(unreachable_patterns)]
    fn set_word_unchecked(&mut self, addr: Addr, word: Word) {
        if addr >= MEM_MAPPED_START_ADDR {
            macro_rules! devices {
                ($($dev:ty),*) => {
                    match addr {
                        $(<$dev as MemMapped>::ADDR => self.set_device_reg::<$dev>(word).unwrap(),)*
                        _ => (), // unimplemented!() // TODO: make a sane handler?
                    }
                };
            }

            devices!(
                KBSR, KBDR,
                DSR, DDR,
                BSP, PSR, MCR,
                G0CR, G0DR, G1CR, G1DR, G2CR, G2DR, G3CR, G3DR, G4CR, G4DR, G5CR, G5DR, G6CR, G6DR, G7CR, G7DR,
                A0CR, A0DR, A1CR, A1DR, A2CR, A2DR, A3CR, A3DR, A4CR, A4DR, A5CR, A5DR,
                P0CR, P0DR, P1CR, P1DR,
                CLKR,
                T0CR, T0DR, T1CR, T1DR
            )
        } else {
            self.set_word_force_memory_backed(addr, word)
        }
    }

    #[forbid(unreachable_patterns)]
    fn get_word_unchecked(&self, addr: Addr) -> Word {
        if addr >= MEM_MAPPED_START_ADDR {
            // TODO: mem mapped peripherals!
            macro_rules! devices {
                ($($dev:ty),*) => {
                    match addr {
                        $(<$dev as MemMapped>::ADDR => *self.get_device_reg::<$dev>().unwrap(),)*
                        // $(devices!( $($special_access)? $dev ))*
                        _ => 0, // unimplemented!() // TODO: make a sane handler?
                    }
                };
            }

            devices!(
                KBSR, KBDR,
                DSR, DDR,
                BSP, PSR, MCR,
                G0CR, G0DR, G1CR, G1DR, G2CR, G2DR, G3CR, G3DR, G4CR, G4DR, G5CR, G5DR, G6CR, G6DR, G7CR, G7DR,
                A0CR, A0DR, A1CR, A1DR, A2CR, A2DR, A3CR, A3DR, A4CR, A4DR, A5CR, A5DR,
                P0CR, P0DR, P1CR, P1DR,
                CLKR,
                T0CR, T0DR, T1CR, T1DR
            )
        } else {
            self.get_word_force_memory_backed(addr)
        }
    }

    fn set_word_force_memory_backed(&mut self, addr: Addr, word: Word) {
        self.memory.write_word(addr, word)
    }

    fn get_word_force_memory_backed(&self, addr: Addr) -> Word {
        self.memory.read_word(addr)
    }

    fn get_machine_state(&self) -> MachineState {
        self.state
    }

    fn reset(&mut self) {
        // TODO: Reset Vector
        // On start!
        // PC = 0x200
        // All regs are 0
        // cc = z
        // pri = 7
        // MCR = 0;
        self.pc = lc3_isa::OS_START_ADDR;

        // Reset memory _before_ setting the PSR and MCR so we don't wipe out
        // their values.
        self.memory.reset();

        self.get_special_reg::<PSR>().set_priority(self, 7);
        self.get_special_reg::<MCR>().run(self);
        self.set_cc(0);

        self.regs = [0; Reg::NUM_REGS];

        self.reset_peripherals();
        self.state = MachineState::Running;

        self.error.set(None);
        self.call_stack = CallStack::new();
    }

    fn halt(&mut self) {
        if self.get_special_reg::<MCR>().is_running() {
            self.get_special_reg::<MCR>().halt(self);
        }

        self.state = MachineState::Halted;
    }

    fn set_error(&self, err: Error) {
        self.error.set(Some(err));
    }

    fn get_error(&self) -> Option<Error> {
        self.error.take()
    }

    fn get_call_stack(&self) -> [Option<(Addr, ProcessorMode)>; MAX_CALL_STACK_DEPTH] {
        self.call_stack.stack
    }

    fn get_call_stack_depth(&self) -> u64 {
        self.call_stack.depth
    }

    fn commit_page(&mut self, page_idx: PageIndex, page: &[Word; PAGE_SIZE_IN_WORDS as usize]) {
        self.memory.commit_page(page_idx, page)
    }

    fn get_program_metadata(&self) -> ProgramMetadata {
        self.memory.get_program_metadata()
    }

    fn set_program_metadata(&mut self, metadata: ProgramMetadata) {
        self.memory.set_program_metadata(metadata)
    }

    fn type_id() -> TypeId {
        TypeId::of::<Interpreter<lc3_traits::memory::MemoryStub, lc3_traits::peripherals::stubs::PeripheralsStub>>()
    }
}

// struct Interpter<'a, M: Memory, P: Periperals<'a>> {
//     memory: M,
//     peripherals: P,
//     regs: [Word; 8],
//     pc: Word,
//     _p: PhantomData<&'a ()>,
// }

// impl<'a, M: Memory, P: Peripherals<'a>> Default for Interpter<'a, M, P> {
//     fn default() -> Self {
//         Self {
//             memory:
//         }
//     }
// }
