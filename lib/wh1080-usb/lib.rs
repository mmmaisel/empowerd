extern crate bytes;
extern crate hidapi;
#[macro_use]
extern crate slog;

use bytes::BytesMut;
use hidapi::HidApi;
use slog::Logger;

struct WH1080Client {
    buffer: BytesMut,
    logger: Option<Logger>,
}

impl WH1080Client {
    const BUFFER_SIZE: usize = 512;

    pub fn new(logger: Option<Logger>) -> WH1080Client {
        return WH1080Client {
            buffer: BytesMut::with_capacity(WH1080Client::BUFFER_SIZE),
            logger: logger,
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

    pub fn read_data(&mut self) -> Result<(), String> {
        let api = match HidApi::new() {
            Ok(x) => x,
            Err(e) => return Err(format!("Error initialising hidapi: {}", e)),
        };

        let device = match api.open(0x1941, 0x8021) {
            Ok(x) => x,
            Err(e) => return Err(format!("Error opening device: {}", e)),
        };

        for _ in 0..8 {
            self.buffer.clear();
            // TODO: do this without unsafe
            unsafe {
                self.buffer.set_len(WH1080Client::BUFFER_SIZE);
            }
            //            self.buffer.resize(WH1080Client::BUFFER_SIZE, 0);
            let num_recv = match device.read(&mut self.buffer) {
                Ok(x) => x,
                Err(e) => return Err(format!("Error reading device: {}", e)),
            };
            unsafe {
                self.buffer.set_len(num_recv);
            }
            //            self.buffer.truncate(num_recv);
            println!(
                "Received {} bytes: {}",
                num_recv,
                String::from_utf8_lossy(&self.buffer)
            );
            println!("Buffer capacity is {}", self.buffer.capacity());
        }

        return Ok(());
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[cfg_attr(test, macro_use)]
extern crate byte_strings;
