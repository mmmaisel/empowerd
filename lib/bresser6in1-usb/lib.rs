/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2021 Max Maisel

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
\******************************************************************************/
#![forbid(unsafe_code)]
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

mod parser;
use parser::*;

mod data;
pub use data::Data;

pub struct Client {
    buffer: BytesMut,
    logger: Option<Logger>,
    parser: Parser,
}

pub const VID: u16 = 0x1941;
pub const PID: u16 = 0x8021;

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

        for device in api.device_list() {
            if device.vendor_id() == VID && device.product_id() == PID {
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

        let device = match api.open(VID, PID) {
            Ok(x) => x,
            Err(e) => return Err(format!("Error opening device: {}", e)),
        };

        for _ in 0..16 {
            self.buffer.clear();
            self.buffer.resize(Client::BUFFER_SIZE, 0);
            let num_recv = match device.read(&mut self.buffer) {
                Ok(x) => x,
                Err(e) => return Err(format!("Error reading device: {}", e)),
            };
            self.buffer.truncate(num_recv);
            if let Some(logger) = &self.logger {
                trace!(
                    logger,
                    "Received {} bytes: {}",
                    num_recv,
                    String::from_utf8_lossy(&self.buffer)
                );
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
                return Data::from_string(&payload);
            }
        }

        return Err(
            "Did not received valid data after several tries, giving up."
                .to_string(),
        );
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[cfg_attr(test, macro_use)]
extern crate byte_strings;
