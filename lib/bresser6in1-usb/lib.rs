extern crate bytes;
extern crate hidapi;
#[macro_use]
extern crate slog;

use std::io::Cursor;

use bytes::BytesMut;
use hidapi::HidApi;
use slog::Logger;

mod bresser6in1_buf;
use bresser6in1_buf::Bresser6in1Buf;

mod message;
use message::Message;

mod parser;
use parser::*;

mod data;
use data::*;

struct Client {
    buffer: BytesMut,
    logger: Option<Logger>,
    parser: Parser,
}

impl Client {
    const BUFFER_SIZE: usize = 512;

    pub fn new(logger: Option<Logger>) -> Client {
        return Client {
            buffer: BytesMut::with_capacity(Client::BUFFER_SIZE),
            logger: logger,
            parser: Parser::new(),
        };
    }

    pub fn device_info(&self) -> Result<String, String> {
        let api = match HidApi::new() {
            Ok(x) => x,
            Err(e) => return Err(format!("Error initialising hidapi: {}", e)),
        };

        for device in api.devices().into_iter() {
            if device.vendor_id == 0x1941 && device.product_id == 0x8021 {
                return Ok(format!("Found matching device {:X?}", device));
            }
        }

        return Err("No matching device found.".to_string());
    }

    pub fn read_data(&mut self) -> Result<Data, String> {
        let api = match HidApi::new() {
            Ok(x) => x,
            Err(e) => return Err(format!("Error initialising hidapi: {}", e)),
        };

        let device = match api.open(0x1941, 0x8021) {
            Ok(x) => x,
            Err(e) => return Err(format!("Error opening device: {}", e)),
        };

        for _ in 0..16 {
            self.buffer.clear();
            // TODO: do this without unsafe
            unsafe { self.buffer.set_len(Client::BUFFER_SIZE); }
            //            self.buffer.resize(Client::BUFFER_SIZE, 0);
            let num_recv = match device.read(&mut self.buffer) {
                Ok(x) => x,
                Err(e) => return Err(format!("Error reading device: {}", e)),
            };
            unsafe { self.buffer.set_len(num_recv); }
            //            self.buffer.truncate(num_recv);
            if let Some(logger) = &self.logger {
                trace!(logger, "Received {} bytes: {}", num_recv,
                    String::from_utf8_lossy(&self.buffer));
                #[cfg(debug_assertions)]
                trace!(logger, "Buffer capacity is {}", self.buffer.capacity());
            }

            let msg = Cursor::new(&mut self.buffer).to_message()?;
            let result = match self.parser.parse_message(msg) {
                Ok(x) => x,
                Err(e) => {
                    if let ParserError::Error(err) = e {
                        return Err(err);
                    }
                    continue;
                }
            };

            if let ParserResult::Success(payload) = result {
                return Data::from_string(payload);
            }
        }

        return Err(
            "Did not received valid data after several tries, giving up.".
            to_string())
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[cfg_attr(test, macro_use)]
extern crate byte_strings;
