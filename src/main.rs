extern crate serialport;
extern crate byteorder;

use std::io::{self, Write};
use std::time::Duration;
use byteorder::{ByteOrder, LittleEndian};
use serialport::prelude::*;

pub struct SdsData {
    pm2_5 : u16,
    pm10 : u16,
}

struct SdsDataPacket {
    pm2_5 : u16,
    pm10 : u16,
    checksum : u8,
    tail : u8
}

impl From<&[u8; 8]> for SdsDataPacket {
    fn from(bytes: &[u8; 8]) -> Self {
        SdsDataPacket {
            pm2_5: LittleEndian::read_u16(&bytes[0..2]),
            pm10: LittleEndian::read_u16(&bytes[2..4]),
            checksum: bytes[6],
            tail: bytes[7]
        }
    }
}

fn main() {
    let port_name = "COM17";

    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(10);
    settings.baud_rate = 9600;

    let mut port = match serialport::open_with_settings(&port_name, &settings) {
        Ok(mut port) => port,
        Err(e) => {
            eprintln!(
                "Failed to open \"{}\". Error: {}",
                port_name,
                e
            );
            ::std::process::exit(1);
        }
    };

    let mut last_two_bytes = [0; 2];
    loop {
        last_two_bytes[0] = last_two_bytes[1];
        match port.read_exact(&mut last_two_bytes[1..2]) {
            Ok(t) => (),
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        }

        if last_two_bytes == [0xAA, 0xC0] {
            let mut data_bytes = [0; 8];
            match port.read_exact(&mut data_bytes) {
                Ok(t) => {
                    let packet = SdsDataPacket::from(&data_bytes);
                    let pm2_5 = packet.pm2_5 as f64;
                    let pm10 = packet.pm10 as f64;
                    let corrected = (pm2_5 / 10.0) / ((-0.509_f64 *((pm10 / pm2_5).ln())) + 1.2203_f64);
                    println!("pm2.5: {}, pm10: {}, corrected pm2.5: {}", (packet.pm2_5 as f32)/10.0, (packet.pm10 as f32)/10.0, corrected);
                },
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                Err(e) => eprintln!("{:?}", e),
            }
        }
    }
}
