//! TODO!

use crate::io_peripherals::{InputSink, OutputSource};

use lc3_shims::peripherals::{
    AdcShim, ClockShim, GpioShim, InputShim, OutputShim, PwmShim, TimersShim,
};
use lc3_shims::peripherals::{ShareablePeripheralsShim, Sink, Source};
use lc3_traits::peripherals::{PeripheralSet, Peripherals};

use std::sync::{Arc, Mutex, RwLock};

pub type ShimPeripheralSet<'io> = ShareablePeripheralsShim<'io>;

pub struct Shims {
    pub gpio: Arc<RwLock<GpioShim>>,
    pub adc: Arc<RwLock<AdcShim>>,
    pub pwm: Arc<Mutex<PwmShim>>,
    pub timers: Arc<Mutex<TimersShim>>,
    pub clock: Arc<RwLock<ClockShim>>,
}

pub fn new_shim_peripherals_set<'io, I, O>(
    input: &'io I,
    output: &'io O,
) -> (
    ShimPeripheralSet<'io>,
    &'io impl InputSink,
    &'io impl OutputSource,
)
where
    I: InputSink + Source + Send + Sync + 'io,
    O: OutputSource + Sink + Send + Sync + 'io,
{
    let gpio_shim = Arc::new(RwLock::new(GpioShim::default()));
    let adc_shim = Arc::new(RwLock::new(AdcShim::default()));
    let pwm_shim = Arc::new(Mutex::new(PwmShim::default()));
    let timer_shim = Arc::new(Mutex::new(TimersShim::default()));
    let clock_shim = Arc::new(RwLock::new(ClockShim::default()));

    let input_shim = Arc::new(Mutex::new(InputShim::with_ref(input)));
    let output_shim = Arc::new(Mutex::new(OutputShim::with_ref(output)));

    (
        PeripheralSet::new(
            gpio_shim,
            adc_shim,
            pwm_shim,
            timer_shim,
            clock_shim,
            input_shim,
            output_shim,
        ),
        input,
        output,
    )
}

impl Shims {
    pub fn from_peripheral_set<'io>(p: &ShimPeripheralSet<'io>) -> Self {
        Self {
            gpio: p.get_gpio().clone(),
            adc: p.get_adc().clone(),
            pwm: p.get_pwm().clone(),
            timers: p.get_timers().clone(),
            clock: p.get_clock().clone(),
        }
    }
}


// TODO: why does this exist as more than a type alias?
