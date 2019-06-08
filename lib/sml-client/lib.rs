extern crate bytes;
extern crate crc16;
extern crate serialport;

use std::io::Cursor;
use std::time::Duration;
use std::thread;

use bytes::BytesMut;
use serialport::{SerialPort, SerialPortSettings};

mod doc;
use doc::SmlBody;
use doc::SmlFile;
use doc::SmlStream;

pub struct SmlClient
{
    port: Box<SerialPort>,
    buffer: BytesMut
}

impl SmlClient
{
    const BUFFER_SIZE: usize = 512;

    const OBIS_CONSUMED: [u8; 6] = [1, 0, 1, 8, 0, 255];
    const OBIS_PRODUCED: [u8; 6] = [1, 0, 2, 8, 0, 255];

    pub fn new(port_name: String, baudrate: u32)
        -> Result<SmlClient, String>
    {
        let mut settings = SerialPortSettings::default();
        settings.baud_rate = baudrate;
        // TODO: open port on demand, allow running without serial device
        let mut port = match serialport::open_with_settings(
            &port_name, &settings)
        {
            Ok(port) => port,
            Err(e) => return Err(format!(
                "Failed to open {}, error: {}", port_name, e))
        };

        // TODO: remove this
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

        // TODO: log this if logger is given
        if cfg!(debug_assertions)
        {
            println!("data: {:?}", self.buffer);
            println!("cap: {}", self.buffer.capacity());
        }
        let mut cursor = Cursor::new(&mut self.buffer);
        let mut streams = SmlStream::deserialize(&mut cursor)?;

        return match streams.pop()
        {
            Some(x) => SmlFile::deserialize(x),
            None => Err("No data received".to_string())
        };
    }

    pub fn extract_produced_consumed(file: SmlFile)
        -> Result<(f64, f64), String>
    {
        let message = match file.messages.iter().find(|elem|
        {
            if let SmlBody::GetListResponse(_) = elem.body
            {
                return true;
            }
            return false;
        }){
            Some(x) => x,
            None => return Err(
                "No SmlGetListResponse found in SmlFile".to_string())
        };

        let values = match &message.body
        {
            SmlBody::GetListResponse(x) => &x.values,
            _ => return Err(
                "No SmlGetListResponse found in SmlMessage".to_string())
        };

        return values.iter().try_fold((0.0, 0.0), |mut acc, val|
        {
            if val.obj_name == SmlClient::OBIS_CONSUMED
            {
                acc.0 = match val.as_f64()
                {
                    Some(x) => x,
                    None => return Err(
                        "Could not convert SML data to number".to_string())
                };
            }
            else if val.obj_name == SmlClient::OBIS_PRODUCED
            {
                acc.1 = match val.as_f64()
                {
                    Some(x) => x,
                    None => return Err(
                        "Could not convert SML data to number".to_string())
                };
            }
            Ok(acc)
        });
    }

    pub fn get_consumed_produced(&mut self)
        -> Result<(f64, f64), String>
    {
        let data = self.get_sml_file()?;
        return SmlClient::extract_produced_consumed(data);
    }
}

#[cfg(test)]
mod tests;
