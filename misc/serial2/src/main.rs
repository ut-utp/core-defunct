
use std::{time::Duration, borrow::Cow};
use std::path::PathBuf;
use std::io::{BufRead, BufReader};

use serialport::{DataBits, FlowControl, Parity, StopBits, SerialPortType, SerialPortInfo};
use structopt::StructOpt;


#[derive(structopt::StructOpt, Debug)]
struct Args {
    #[structopt(short = "p", long = "device-path", parse(from_os_str))]
    device_path: Option<PathBuf>,

    #[structopt(short = "b", long = "baud-rate", default_value = "1500000")]
    baud_rate: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Args { device_path, baud_rate } = dbg!(Args::from_args());

    let device_path: Cow<_> = if let Some(ref device_path) = device_path {
        device_path.to_str().unwrap().into()
    } else {
        let available_ports = serialport::available_ports().expect("no device path explicitly specified, couldn't detect available device");
        let found_port = available_ports
            .into_iter()
            .filter(|p| matches!(p, SerialPortInfo { port_type: SerialPortType::UsbPort(_) , ..}))
            .nth(1)
            .expect("no device path explicitly specified, couldn't find a USB Serial device...");

        eprintln!("using USB port: {found_port:#?}");
        found_port.port_name.into()
    };

    let dev = serialport::new(device_path, baud_rate)
        .data_bits(DataBits::Eight)
        .flow_control(FlowControl::None)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .timeout(Duration::from_secs(2))
        .open_native()?;

    let mut dev = BufReader::new(dev);

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

    let mut buf = String::new();
    loop {
        match dev.read_line(&mut buf) {
            Ok(n) => print!("[{n:3}] {}", buf),
            Err(err) => eprintln!("error: {err:?}"),
        }
        buf.clear();
    }
}
