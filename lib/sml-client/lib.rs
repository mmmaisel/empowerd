extern crate bytes;
extern crate crc16;
extern crate serialport;

use std::io::Cursor;
use std::time::Duration;
use std::thread;

use bytes::BytesMut;
use serialport::{SerialPort, SerialPortSettings};

mod doc;
use doc::SmlFile;
use doc::SmlBuf;

pub struct SmlClient
{
    port: Box<SerialPort>,
    buffer: BytesMut
}

impl SmlClient
{
    const BUFFER_SIZE: usize = 512;

    pub fn new(port_name: String, baudrate: u32)
        -> Result<SmlClient, String>
    {
        let mut settings = SerialPortSettings::default();
        settings.baud_rate = baudrate;
        let mut port = match serialport::open_with_settings(&port_name, &settings)
        {
            Ok(port) => port,
            Err(e) => return Err(format!(
                "Failed to open {}, error: {}", port_name, e))
        };

        port.write_request_to_send(false);

        return Ok(SmlClient
        {
            port: port,
            buffer: BytesMut::with_capacity(SmlClient::BUFFER_SIZE)
        });
    }

    pub fn get_sml_file(&mut self) -> Result<SmlFile, String>
    {
        if let Err(e) = self.port.clear(serialport::ClearBuffer::Input)
        {
            return Err(
                format!("Failed to flush input buffer, error: {}", e));
        }
        thread::sleep(Duration::from_secs(3));

        self.buffer.clear();
        // TODO: do this without unsafe
        unsafe { self.buffer.set_len(SmlClient::BUFFER_SIZE); }
        let num_recv = match self.port.read(&mut self.buffer)
        {
            Ok(x) => x,
            Err(e) => return Err(
                format!("Reading data failed, error: {}", e))
        };
        unsafe { self.buffer.set_len(num_recv); }

        let mut cursor = Cursor::new(&mut self.buffer);
        let mut files = SmlFile::deserialize_stream(&mut cursor)?;

        return match files.pop()
        {
            Some(x) => Ok(x),
            None => Err("No data received".to_string())
        };
    }

    pub fn extract_produced_consumed(data: SmlFile)
        -> Result<(u64, u64), String>
    {
        return Err(format!("not implemented, data is {:?}", data));
    }

    pub fn get_consumed_produced(&mut self)
        -> Result<(u64, u64), String>
    {
        let data = self.get_sml_file()?;
        return SmlClient::extract_produced_consumed(data);
    }
}
