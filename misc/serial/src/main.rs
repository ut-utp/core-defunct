
use std::{time::Duration, borrow::Cow};
use std::path::PathBuf;

use tokio_serial::{DataBits, FlowControl, Parity, StopBits, SerialPortType, SerialPortInfo, SerialPortBuilderExt, SerialPort};
use tokio::io::AsyncReadExt;
use structopt::StructOpt;


#[derive(structopt::StructOpt, Debug)]
struct Args {
    #[structopt(short = "p", long = "device-path", parse(from_os_str))]
    device_path: Option<PathBuf>,

    #[structopt(short = "b", long = "baud-rate", default_value = "4000000")]
    baud_rate: u32,
}

// const BAUD_RATE: u32 = 1_500_000;
// const BAUD_RATE: u32 = 15200;
// const BAUD_RATE: u32 = 115_200;
// const BAUD_RATE: u32 = 2_300_000;
// const BAUD_RATE: u32 = 2_500_000;
// const BAUD_RATE: u32 = 2_500_000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Args { device_path, baud_rate } = dbg!(Args::from_args());

    let device_path: Cow<_> = if let Some(ref device_path) = device_path {
        device_path.to_str().unwrap().into()
    } else {
        let available_ports = tokio_serial::available_ports().expect("no device path explicitly specified, couldn't detect available device");
        let found_port = available_ports
            .into_iter()
            .filter(|p| matches!(p, SerialPortInfo { port_type: SerialPortType::UsbPort(_) , ..}))
            .nth(1)
            .expect("no device path explicitly specified, couldn't find a USB Serial device...");

        eprintln!("using USB port: {found_port:#?}");
        found_port.port_name.into()
    };

    let mut dev = tokio_serial::new(device_path, 9600)
        // .data_bits(DataBits::Eight)
        // .flow_control(FlowControl::None)
        // .parity(Parity::None)
        // .stop_bits(StopBits::One)
        // .timeout(Duration::from_secs(2))
        .open_native_async()?;

    dev.set_baud_rate(baud_rate).unwrap();

    // let settings = SerialPortSettings {
    //     baud_rate,
    //     data_bits: DataBits::Eight,
    //     flow_control: FlowControl::None,
    //     // parity: Parity::Even,
    //     parity: Parity::None,
    //     stop_bits: StopBits::One,
    //     timeout: Duration::from_secs(100),
    // };

    // let mut dev = Serial::from_path(device_path, &settings)?;

    loop {
        print!("{}", dev.read_u8().await? as char);
    }
}