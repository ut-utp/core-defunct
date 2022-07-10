//! TODO!

use super::{BlackBox, Init};
use crate::shim_support::Shims;

use lc3_shims::peripherals::SourceShim;
use lc3_traits::control::rpc::{
    futures::SyncEventFutureSharedState,
    Controller, RequestMessage, ResponseMessage,
};
use lc3_device_support::{
    rpc::{
        transport::uart_host::{HostUartTransport, SerialPortBuilder},
        encoding::{PostcardEncode, PostcardDecode, Cobs},
    },
    util::Fifo,
};

use std::{
    borrow::Cow,
    sync::Mutex,
    thread::Builder as ThreadBuilder,
    path::{Path, PathBuf},
    default::Default,
    marker::PhantomData,
};

// Static data that we need:
// TODO: note that, like sim and sim_rpc, this will cause problems if more than
// 1 instance of this simulator is instantiated.
lazy_static::lazy_static! {
    pub static ref EVENT_FUTURE_SHARED_STATE_CONT: SyncEventFutureSharedState =
        SyncEventFutureSharedState::new();
}

type Cont<'ss, EncFunc: FnMut() -> Cobs<Fifo<u8>>> = Controller<
    'ss,
    HostUartTransport,
    SyncEventFutureSharedState,
    RequestMessage,
    ResponseMessage,
    PostcardEncode<RequestMessage, Cobs<Fifo<u8>>, EncFunc>,
    PostcardDecode<ResponseMessage, Cobs<Fifo<u8>>>,
>;

// #[derive(Debug)]
pub struct BoardDevice<'ss, EncFunc = Box<dyn FnMut() -> Cobs<Fifo<u8>>>>
where
    EncFunc: FnMut() -> Cobs<Fifo<u8>>,
{
    controller: Cont<'ss, EncFunc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SerialSettings {
    DefaultsWithBaudRate { baud_rate: u32, path: String, },
    Custom(SerialPortBuilder),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardConfig {
    pub settings: SerialSettings,
}

// TODO: use Strings instead?
impl Default for BoardConfig {
    fn default() -> Self {
        #[cfg(target_os = "windows")]
        { Self::detect_windows() }

        #[cfg(target_os = "macos")]
        { Self::detect_macos() }

        #[cfg(target_os = "linux")]
        { Self::detect_linux() }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        { Self::new("/dev/tm4c", 1_500_000) }
    }
}

// TODO: port over [this script]
//
// [this script](https://github.com/ut-ras/Rasware/blob/f3750ff0b8f7f7791da2fee365462d4c78f62a49/RASLib/detect-board)

impl BoardConfig {
    #[cfg_attr(all(docs, not(doctest)), doc(cfg(target_os = "windows")))]
    #[cfg(target_os = "windows")]
    pub fn detect_windows() -> Self {
        unimplemented!()
    }

    #[cfg_attr(all(docs, not(doctest)), doc(cfg(target_os = "linux")))]
    #[cfg(target_os = "linux")]
    pub fn detect_linux() -> Self {
        Self::new("/dev/lm4f", 1_500_000)
    }

    #[cfg_attr(all(docs, not(doctest)), doc(cfg(target_os = "macos")))]
    #[cfg(target_os = "macos")]
    pub fn detect_macos() -> Self {
        unimplemented!()
    }
}

impl BoardConfig {
    pub /*const*/ fn new<'a>(path: impl Into<Cow<'a, str>>, baud_rate: u32) -> Self {
        Self {
            settings: SerialSettings::DefaultsWithBaudRate { baud_rate, path: path.into().to_string() }
        }
    }

    pub /*const*/ fn new_with_config(config: SerialPortBuilder) -> Self {
        Self {
            settings: SerialSettings::Custom(config),
        }
    }
}

impl BoardConfig {
    fn new_transport(self) -> HostUartTransport {
        // Note: we unwrap here! This is probably not great!
        // (TODO)
        match self.settings {
            SerialSettings::DefaultsWithBaudRate { path, baud_rate } => {
                HostUartTransport::new(path, baud_rate).unwrap()
            },

            SerialSettings::Custom(config) => {
                HostUartTransport::new_with_config(config).unwrap()
            },
        }
    }
}

impl<'s> Init<'s> for BoardDevice<'static, Box<dyn FnMut() -> Cobs<Fifo<u8>>>>
where
    BoardConfig: Default,
{
    type Config = BoardConfig;

    // Until we get existential types (or `impl Trait` in type aliases) we can't
    // just say a type parameter is some specific type that we can't name (i.e.
    // a particular function).
    //
    // Instead we have to name it explicitly, so we do this unfortunate hack:
    type ControlImpl = Cont<'static, Box<dyn FnMut() -> Cobs<Fifo<u8>>>>;

    type Input = SourceShim; // TODO
    type Output = Mutex<Vec<u8>>; // TODO

    fn init_with_config(
        b: &'s mut BlackBox,
        config: BoardConfig,
    ) -> (
        &'s mut Self::ControlImpl,
        Option<Shims<'static>>,
        Option<&'s Self::Input>,
        Option<&'s Self::Output>,
    ) {
        let func: Box<dyn FnMut() -> Cobs<Fifo<u8>>> = Box::new(|| Cobs::try_new(Fifo::new()).unwrap());

        let controller = Controller::new(
            PostcardEncode::new(func),
            PostcardDecode::new(),
            config.new_transport(),
            &*EVENT_FUTURE_SHARED_STATE_CONT
        );

        let storage: &'s mut _ = b.put(BoardDevice::<_> { controller });

        (
            &mut storage.controller,
            None,
            None, // TODO
            None, // TODO
        )
    }
}
