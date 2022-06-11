//! Controller side for the [`Control`](super::Control) RPC set up.
//!
//! TODO!

// TODO: auto gen (proc macro, probably) the crimes below from the `Control`
// trait.

use super::{State, Event, Control, Transport};
use super::messages::{RequestMessage, ResponseMessage};
use super::encoding::{Encode, Decode, Transparent};
use super::futures::{EventFutureSharedStatePorcelain, EventFuture};
use crate::control::control::{
    MAX_BREAKPOINTS, MAX_MEMORY_WATCHPOINTS, MAX_CALL_STACK_DEPTH,
    ProcessorMode, Idx
};
use crate::control::load::{
    LoadApiSession, CHUNK_SIZE_IN_WORDS, PageWriteStart, PageIndex, Offset,
    StartPageWriteError, PageChunkError, FinishPageWriteError
};
use crate::control::{ProgramMetadata, DeviceInfo, UnifiedRange};
use crate::error::Error as Lc3Error;
use crate::peripherals::{
    adc::{AdcPinArr, AdcState, AdcReadError},
    gpio::{GpioPinArr, GpioState, GpioReadError, GpioBank},
    pwm::{PwmPinArr, PwmState},
    timers::{TimerArr, TimerMode, TimerState},
};

use lc3_isa::{Reg, Addr, Word};

use core::cell::RefCell;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};

// Converts calls on the control interface to messages and sends said messages.
//
// Sends Requests and receives Responses.
//
// As mentioned elsewhere, there's a level of indirection between
// `RequestMessage`/`ResponseMessage` and the types used here so that users can
// experiment with their own message types. This is probably a moot point since
// you can already do this by defining an encoding layer that does the
// conversion for you. The only thing the below buys you is being able to use
// message types that don't implement Debug. Update: this is no longer true. (TODO)
#[derive(Debug)]
pub struct Controller<
    'a,
    T,
    S,
    Req = RequestMessage,
    Resp = ResponseMessage,
    ReqEnc = Transparent<RequestMessage>,
    RespDec = Transparent<ResponseMessage>,
>
where
    Req: Debug,
    Resp: Debug,
    RequestMessage: Into<Req>,
    Resp: Into<ResponseMessage>,
    ReqEnc: Encode<Req>,
    RespDec: Decode<Resp>,
    T: Transport<<ReqEnc as Encode<Req>>::Encoded, <RespDec as Decode<Resp>>::Encoded>,
    S: EventFutureSharedStatePorcelain,
{
    _encoded_formats: PhantomData<(Req, Resp)>,
    pub transport: T,
    enc: RefCell<ReqEnc>,
    dec: RefCell<RespDec>,
    // pending_messages: Cell<[Option<ControlMessage>; 2]>,
    // pending_messages: [Option<ControlMessage>; 2],
    shared_state: &'a S,
    waiting_for_event: AtomicBool, // TODO: no reason for this to be Atomic // Note: it's atomic so we can maintain interior mutability?

    #[cfg_attr(all(docs, not(doctest)), doc(cfg(feature = "std")))]
    #[cfg(feature = "std")]
    retry_timeout: Option<std::time::Duration>,
}

// TODO: make a builder!

impl<'a, Req, Resp, E, D, T, S> Controller<'a, T, S, Req, Resp, E, D>
where
    Req: Debug,
    Resp: Debug,
    RequestMessage: Into<Req>,
    Resp: Into<ResponseMessage>,
    E: Encode<Req>,
    D: Decode<Resp>,
    T: Transport<<E as Encode<Req>>::Encoded, <D as Decode<Resp>>::Encoded>,
    S: EventFutureSharedStatePorcelain,
{
    // When const functions can be in blanket impls, this can be made `const`.
    //
    // Note: we take `decode` and `encode` as parameters here even though the
    // actual value is never used so that users don't have to resort to using
    // the turbofish syntax to specify what they want the encoding layer to be.
    pub /*const*/ fn new(enc: E, dec: D, transport: T, shared_state: &'a S) -> Self {
        let c = Self {
            // encoding,
            _encoded_formats: PhantomData,
            enc: RefCell::new(enc),
            dec: RefCell::new(dec),
            transport,
            // pending_messages: Cell::new([None; 2]),
            // pending_messages: [None; 2],
            shared_state,
            waiting_for_event: AtomicBool::new(false),

            #[cfg_attr(all(docs, not(doctest)), doc(cfg(feature = "std")))]
            #[cfg(feature = "std")]
            retry_timeout: None,
        };

        // In case we've just attached to a running interpreter:
        //
        // (we do this check once)
        //
        // TODO: perf hit? do we want to move this out to a separate function that
        // needs to be called explicitly?
        if let State::RunningUntilEvent = c.get_state() {
            c.waiting_for_event.store(true, Ordering::SeqCst);

            // we'd like to do this but can't because of the multi-batch issue...
            //
            // instead users of this `Control` impl will just need to
            // call `run_until_event` themselves!
            // c.shared_state.add_new_future().unwrap();
        }

        c
    }

    using_std! {
        // TODO: explain better, esp the potential downsides (repeated
        // messages!)
        //
        /// If enabled, retries sending the request messages for a response
        /// we're waiting on after the given timeout.
        ///
        /// This can lead to the device getting duplicated request messages; for
        /// commands that are not idempotent (i.e. Step) this will be
        /// observable.
        ///
        /// The upside is that this enables the `Controller` to "recover" from
        /// errors like the board being restarted in a way that's not aligned
        /// with req/resps.
        pub fn with_retry_timeout(
            mut self,
            timeout: std::time::Duration,
        ) -> Self {
            self.retry_timeout = Some(timeout);
            self
        }
    }
}

// TODO: this is a stopgap; eventually we should have an error variant on the
// req/resp message enums. See the notes in `device-support/src/rpc/mod.rs` for
// more.

// TODO: when the Try trait goes stable, impl it on this type or something like it.
/*#[derive(Debug, Clone)]
enum TickAttempt<M, D, T> {
    NoMessage,
    Message(M),
    DecodeError(D),
    TransportError(T),
}*/

// Until we have Try this is more ergonomic:
enum TickError<DecErr, TranspErr> {
    DecodeError(DecErr),
    TransportError(TranspErr),
}

type TickAttempt<M, D, T> = Result<M, Option<TickError<D, T>>>;

impl<'a, Req, Resp, E, D, T, S> Controller<'a, T, S, Req, Resp, E, D>
where
    Req: Debug,
    Resp: Debug,
    RequestMessage: Into<Req>,
    Resp: Into<ResponseMessage>,
    E: Encode<Req>,
    D: Decode<Resp>,
    T: Transport<<E as Encode<Req>>::Encoded, <D as Decode<Resp>>::Encoded>,
    S: EventFutureSharedStatePorcelain,
{
    // For now, we're going to assume sequential consistency (we receive
    // responses to messages in the same order we filed the requests). (TODO)
    //
    // Responses to our one non-blocking call (`run_until_event`) are the only
    // thing that could interrupt this.
    //
    // In situations where this is called after sending data and a response is
    // expected `use_blocking_get` can be set to `true`. This will use
    // `get_blocking` on the underlying transport which may save the caller of
    // this function (if waiting on a response) some spinning.
    fn tick_inner(&self, use_blocking_get: bool) -> TickAttempt<ResponseMessage, D::Err, T::RecvErr> {
        let transport_response = if use_blocking_get {
            self.transport.blocking_get()
        } else {
            self.transport.get()
        };
        let encoded_message = transport_response
            .map_err(|e| e.map(TickError::TransportError))?;

        let message = self.dec.borrow_mut()
            .decode(&encoded_message)
            .map_err(|d| Some(TickError::DecodeError(d)))?; // TODO: do better?

        let message = message.into();

        if let ResponseMessage::RunUntilEvent(event) = message {
            if self.waiting_for_event.load(Ordering::SeqCst) {
                // println!("resolving the rpc future"); // TODO: logging w/feature flag
                self.shared_state.resolve_all(event).unwrap();
                self.waiting_for_event.store(false, Ordering::SeqCst);

                /*NoMessage*/ Err(None)
            } else {
                // Something has gone very wrong.
                // We were told an event happened but we never asked.
                unreachable!()
            }
        } else {
            Ok(message)
        }
    }
}


macro_rules! ctrl {
    ($s:ident, $req:expr, $resp:pat$(, $ret:expr)?) => {{
        use RequestMessage::*;
        use ResponseMessage as R;
        let m = $req.into();

        $s.transport.send($s.enc.borrow_mut().encode(&m)).unwrap(); // TODO: don't panic? not sure how we'd realistically deal with any transport errors..

        #[cfg(feature = "std")]
        let msg_sent_at = std::time::Instant::now();

        log::trace!("sending message: {m:?}");

        loop {
            // Because we have just sent a message, we expect a response.
            //
            // This means we can use `blocking_get`.
            match Controller::tick_inner($s, true) {
                // If we got a message, process it:
                Ok(m) => {
                    if let $resp = m {
                        log::trace!("got expected resp: {}", core::stringify!($resp));
                        break $($ret)?
                    } else {
                        // We used to panic here but no longer...
                        //
                        // We're effectively throwing this response message away
                        // and trying again (if we time out). The hope is that
                        // we'll eventually realign ourselves.
                        //
                        // The ideal case is just that the device went away for
                        // a bit (i.e. was paused in GDB) causing us to retry
                        // meaning that we're now getting duplicated respoonses
                        // for the previous request we sent out. In the future
                        // maybe we'll cache the last request we tried and
                        // _ensure_ that the response type we're getting
                        // unexpectedly is that of the previous message.
                        //
                        // But for now we just log the error and move on.
                        log::error!("Incorrect response for message! got: {m:?}, expected: {}", core::stringify!($resp))
                    }
                },

                // If we got no message, try, try again:
                Err(None) => {
                    // TODO: implement some kind of backoff here??
                    log::trace!("no message received (waiting on: {}), asking the transport to check again...", core::stringify!($resp));

                    // Separate change:
                    #[cfg(feature = "std")]
                    {
                        // Try again!
                        //
                        // Hopefully the message actually was lost... otherwise
                        // we'll get two responses and crash.
                        if let Some(retry_timeout) = $s.retry_timeout {
                            if std::time::Instant::now().duration_since(msg_sent_at) > retry_timeout {
                                log::error!("we've hit our retry timeout ({retry_timeout:?})! sending message `{m:?}` again...");

                                $s.transport.send($s.enc.borrow_mut().encode(&m)).unwrap(); // TODO: don't panic? not sure how we'd realistically deal with any transport errors..
                            }
                        }

                        // TODO: we should maybe "poison" the run until event
                        // future here, if we're assuming that the board has
                        // been reset...
                        //
                        // Or not; I think we don't intend to support tolerating
                        // board resets in the general case. If there are no
                        // pending futures at the time of reset it'll __happen__
                        // to work and that's okay.
                        //
                        // Note that if we ever do a caching control middleware,
                        // it'll have a lot of its assumptions violated if the
                        // board resets from under it. That's okay too.
                    }
                },

                // If we got a transport error, bail:
                Err(Some(TickError::TransportError(e))) => panic!("Transport error! `{:?}`", e),

                // If we got a decode error, assume a problem in transmission
                // and try again. (TODO: this is... fraught. but given how rare
                // UART transmission errors are, this is probably good enough)
                Err(Some(TickError::DecodeError(e))) => {
                    log::error!("Decode Error: `{:?}`", e);
                    log::error!("Sending message `{m:?}` again...");

                    // TODO: because send _consumes_ the message we have to do
                    // the encode here. On the one hand having the transport
                    // consume the message should allow for good zero-copy
                    // impls but on the other hand it means we can't cache the
                    // encode in situations like these... Not sure what the
                    // right tradeoff is.
                    $s.transport.send($s.enc.borrow_mut().encode(&m)).unwrap(); // TODO: don't panic?
                }
            }
        }
    }};
}

#[forbid(irrefutable_let_patterns)]
impl<'a, Req, Resp, E, D, T, S> Control for Controller<'a, T, S, Req, Resp, E, D>
where
    Req: Debug,
    Resp: Debug,
    RequestMessage: Into<Req>,
    Resp: Into<ResponseMessage>,
    E: Encode<Req>,
    D: Decode<Resp>,
    T: Transport<<E as Encode<Req>>::Encoded, <D as Decode<Resp>>::Encoded>,
    S: EventFutureSharedStatePorcelain,
{
    type EventFuture = EventFuture<'a, S>;

    fn get_pc(&self) -> Addr { ctrl!(self, GetPc, R::GetPc(addr), addr) }
    fn set_pc(&mut self, addr: Addr) { ctrl!(self, SetPc { addr }, R::SetPc) }

    fn get_register(&self, reg: Reg) -> Word { ctrl!(self, GetRegister { reg }, R::GetRegister(word), word) }
    fn set_register(&mut self, reg: Reg, data: Word) { ctrl!(self, SetRegister { reg, data }, R::SetRegister) }

    fn get_registers_psr_and_pc(&self) -> ([Word; Reg::NUM_REGS], Word, Word) {
        ctrl!(self, GetRegistersPsrAndPc, R::GetRegistersPsrAndPc(r), r)
    }

    fn read_word(&self, addr: Addr) -> Word { ctrl!(self, ReadWord { addr }, R::ReadWord(w), w) }
    fn write_word(&mut self, addr: Addr, word: Word) { ctrl!(self, WriteWord { addr, word }, R::WriteWord) }

    fn start_page_write(&mut self, page: LoadApiSession<PageWriteStart>, checksum: u64) -> Result<LoadApiSession<u8>, StartPageWriteError> {
        ctrl!(self, StartPageWrite { page, checksum }, R::StartPageWrite(r), r)
    }
    fn send_page_chunk(&mut self, offset: LoadApiSession<Offset>, chunk: [Word; CHUNK_SIZE_IN_WORDS as usize]) -> Result<(), PageChunkError> {
        ctrl!(self, SendPageChunk { offset, chunk }, R::SendPageChunk(r), r)
    }
    fn finish_page_write(&mut self, page: LoadApiSession<PageIndex>) -> Result<(), FinishPageWriteError> {
        ctrl!(self, FinishPageWrite { page }, R::FinishPageWrite(r), r)
    }

    fn set_breakpoint(&mut self, addr: Addr) -> Result<Idx, ()> {
        ctrl!(self, SetBreakpoint { addr }, R::SetBreakpoint(r), r)
    }
    fn unset_breakpoint(&mut self, idx: Idx) -> Result<(), ()> {
        ctrl!(self, UnsetBreakpoint { idx }, R::UnsetBreakpoint(r), r)
    }
    fn get_breakpoints(&self) -> [Option<Addr>; MAX_BREAKPOINTS] { ctrl!(self, GetBreakpoints, R::GetBreakpoints(r), r) }
    fn get_max_breakpoints(&self) -> Idx { ctrl!(self, GetMaxBreakpoints, R::GetMaxBreakpoints(r), r) }

    fn set_memory_watchpoint(&mut self, addr: Addr) -> Result<Idx, ()> {
        ctrl!(self, SetMemoryWatchpoint { addr }, R::SetMemoryWatchpoint(r), r)
    }
    fn unset_memory_watchpoint(&mut self, idx: Idx) -> Result<(), ()> {
        ctrl!(self, UnsetMemoryWatchpoint { idx }, R::UnsetMemoryWatchpoint(r), r)
    }
    fn get_memory_watchpoints(&self) -> [Option<(Addr, Word)>; MAX_MEMORY_WATCHPOINTS] { ctrl!(self, GetMemoryWatchpoints, R::GetMemoryWatchpoints(r), r) }
    fn get_max_memory_watchpoints(&self) -> Idx { ctrl!(self, GetMaxMemoryWatchpoints, R::GetMaxMemoryWatchpoints(r), r) }

    fn set_depth_condition(&mut self, condition: UnifiedRange<u64>) -> Result<Option<UnifiedRange<u64>>, ()> {
        ctrl!(self, SetDepthCondition { condition }, R::SetDepthCondition(r), r)
    }
    fn unset_depth_condition(&mut self) -> Option<UnifiedRange<u64>> { ctrl!(self, UnsetDepthCondition, R::UnsetDepthCondition(r), r) }
    fn get_depth(&self) -> Result<u64, ()> { ctrl!(self, GetDepth, R::GetDepth(r), r) }
    fn get_call_stack(&self) -> [Option<(Addr, ProcessorMode)>; MAX_CALL_STACK_DEPTH] {
        ctrl!(self, GetCallStack, R::GetCallStack(r), r)
    }

    // Execution control functions:
    fn run_until_event(&mut self) -> Self::EventFuture {
        // If we're in a sealed batch with pending futures, just crash.
        self.shared_state.add_new_future().expect("no new futures once a batch starts to resolve");

        // If we're already waiting for an event, don't bother sending the
        // request along again.
        //
        // If we're not yet waiting for an event:
        if !self.waiting_for_event.load(Ordering::SeqCst) {
            // Send the message:
            /*
            let m = RequestMessage::RunUntilEvent.into();
            self.transport.send(self.enc.borrow_mut().encode(&m)).unwrap();

            // Wait for the acknowledge:
            //
            // (just like `ctrl!` we can use a `blocking_get` here)
            loop {
                match Controller::tick_inner(self, true) {
                    Ok(m) => if let ResponseMessage::RunUntilEventAck = m {
                        break;
                    } else {
                        panic!("Incorrect response for message! got {m:?}, expected `RunUntilEventAck")
                    }

                    Err(None) => {},

                    Err(Some(TickError::TransportError(e))) => {
                        panic!("Transport Error! {:?}", e)
                    },

                    Err(Some(TickError::DecodeError(e))) => {
                        // Try again!
                        log::error!("Decode Error: {:?}", e);
                        self.transport.send(self.enc.borrow_mut().encode(&m)).unwrap();
                    },
                }
            }
            */

            ctrl!(self, RunUntilEvent, R::RunUntilEventAck);

            self.waiting_for_event.store(true, Ordering::SeqCst);
        }

        // println!("new rpc future");

        EventFuture(self.shared_state)
    }

    fn tick(&mut self) -> usize {
        // Because we basically call tick() on every other function call here, we could
        // probably get away with just doing nothing here in practice.
        // But, checking here as well doesn't hurt.
        //
        // We should never actually get a message here (run until event responses are
        // handled within `Self::tick()`) though.
        // Self::tick(self).unwrap_none(); // when this goes stable, use this, maybe (TODO)
        //
        // Since we have not sent a message, we are not *guaranteed* to get any bytes
        // (as mentioned, a RunUntilEvent response is the only thing we could _possibly_
        // get) so we should not block.
        if let Err(None) = Self::tick_inner(self, false) {
            /* We expect to get nothing here. */
        } else {
            panic!("Controller received a message in tick!")
        }

        // This function can (probably, TODO) be safely _not_ called so we
        // return 0:
        0
    }

    fn step(&mut self) -> Option<Event> { ctrl!(self, Step, R::Step(r), r) }
    fn pause(&mut self) { ctrl!(self, Pause, R::Pause) }

    fn get_state(&self) -> State { ctrl!(self, GetState, R::GetState(r), r) }

    fn reset(&mut self) {
        // For now, we won't force all futures to have resolved on a reset.
        // We're still calling reset here (currently a no-op) because eventually
        // this should advance the batch counter (though that may happen on
        // set_event, rendering this function entirely unnecessary).
        self.shared_state.reset();

        ctrl!(self, Reset, R::Reset)
    }

    fn get_error(&self) -> Option<Lc3Error> { ctrl!(self, GetError, R::GetError(r), r) }

    // I/O Access:
    fn get_gpio_states(&self, bank: GpioBank) -> Option<GpioPinArr<GpioState>> { ctrl!(self, GetGpioStates { bank }, R::GetGpioStates(r), r) }
    fn get_gpio_readings(&self, bank: GpioBank) -> Option<GpioPinArr<Result<bool, GpioReadError>>> { ctrl!(self, GetGpioReadings { bank }, R::GetGpioReadings(r), r) }
    fn get_adc_states(&self) -> AdcPinArr<AdcState> { ctrl!(self, GetAdcStates, R::GetAdcStates(r), r) }
    fn get_adc_readings(&self) -> AdcPinArr<Result<u8, AdcReadError>> { ctrl!(self, GetAdcReadings, R::GetAdcReadings(r), r) }
    fn get_timer_modes(&self) -> TimerArr<TimerMode> { ctrl!(self, GetTimerModes, R::GetTimerModes(r), r) }
    fn get_timer_states(&self) -> TimerArr<TimerState> { ctrl!(self, GetTimerStates, R::GetTimerStates(r), r) }
    fn get_pwm_states(&self) -> PwmPinArr<PwmState> { ctrl!(self, GetPwmStates, R::GetPwmStates(r), r) }
    fn get_pwm_config(&self) -> PwmPinArr<u8> { ctrl!(self, GetPwmConfig, R::GetPwmConfig(r), r) }
    fn get_clock(&self) -> Word { ctrl!(self, GetClock, R::GetClock(r), r) }

    fn get_device_info(&self) -> DeviceInfo { ctrl!(self, GetDeviceInfo, R::GetDeviceInfo(r), r) }

    fn get_program_metadata(&self) -> ProgramMetadata { ctrl!(self, GetProgramMetadata, R::GetProgramMetadata(r), r) }
    fn set_program_metadata(&mut self, metadata: ProgramMetadata) { ctrl!(self, SetProgramMetadata { metadata }, R::SetProgramMetadata) }

    fn id(&self) -> crate::control::metadata::Identifier {
        crate::control::metadata::Identifier::new_from_str_that_crashes_on_invalid_inputs("PROX")
    }
}
