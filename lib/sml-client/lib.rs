extern crate bytes;
extern crate crc16;
extern crate serialport;

use std::io::Cursor;
use std::time::Duration;
use serialport::{SerialPort, SerialPortSettings};

mod doc;
use doc::SmlFile;
use doc::SmlBuf;

pub struct SmlClient
{
    port: Box<SerialPort>
}

impl SmlClient
{
    pub fn new(port_name: String, baudrate: u32, timeout: Duration)
        -> Result<SmlClient, String>
    {
        let mut settings = SerialPortSettings::default();
        settings.baud_rate = baudrate;
        settings.timeout = timeout;
        let port = match serialport::open_with_settings(&port_name, &settings)
        {
            Ok(port) => port,
            Err(e) => return Err(format!(
                "Failed to open {}, error: {}", port_name, e))
        };

        return Ok(SmlClient
        {
            port: port
        });
    }

    // TODO: get sml file from last two seconds or so
    // TODO: extract data from file (to btreemap ?)
}
