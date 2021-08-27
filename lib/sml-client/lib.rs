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

use std::io::Cursor;
use std::time::Duration;

use bytes::BytesMut;
use slog::{trace, Logger};
use tokio::io::AsyncReadExt;
use tokio::time::{sleep, timeout};
use tokio_serial::{SerialPort, SerialPortBuilder, SerialStream};

mod doc;
use doc::SmlBody;
use doc::SmlFile;
use doc::SmlStream;

#[cfg(test)]
mod tests;

// XXX: serialport implement custom poll buffer, do not clear buffer if last message was
// incomplete.
//
pub struct SmlClient {
    settings: SerialPortBuilder,
    buffer: BytesMut,
    logger: Option<Logger>,
}

impl SmlClient {
    const BUFFER_SIZE: usize = 1024;

    const OBIS_CONSUMED: [u8; 6] = [1, 0, 1, 8, 0, 255];
    const OBIS_PRODUCED: [u8; 6] = [1, 0, 2, 8, 0, 255];

    const SML_UNIT_WH: u8 = 0x1E;

    pub fn new(
        port_name: String,
        baudrate: u32,
        logger: Option<Logger>,
    ) -> SmlClient {
        return SmlClient {
            settings: tokio_serial::new(port_name, baudrate),
            buffer: BytesMut::with_capacity(SmlClient::BUFFER_SIZE),
            logger: logger,
        };
    }

    pub async fn get_sml_file(&mut self) -> Result<SmlFile, String> {
        let mut port = match SerialStream::open(&self.settings) {
            Ok(port) => port,
            Err(e) => {
                return Err(format!("Failed to open serial port, error: {}", e))
            }
        };

        //port.write_request_to_send(false);
        if let Err(e) = port.clear(tokio_serial::ClearBuffer::Input) {
            return Err(format!("Failed to flush input buffer, error: {}", e));
        }
        sleep(Duration::from_secs(4)).await;

        self.buffer.clear();
        self.buffer.resize(SmlClient::BUFFER_SIZE, 0);
        let num_recv =
            match timeout(Duration::from_secs(1), port.read(&mut self.buffer))
                .await
            {
                Ok(x) => match x {
                    Ok(y) => y,
                    Err(e) => {
                        return Err(format!(
                            "Reading data failed, error: {}",
                            e
                        ))
                    }
                },
                Err(e) => return Err(e.to_string()),
            };
        self.buffer.resize(num_recv, 0);

        match &self.logger {
            Some(x) => {
                trace!(x, "data: {:?}", self.buffer);
                trace!(x, "cap: {}", self.buffer.capacity());
            }
            None => (),
        }

        // TODO: optimize away cursor
        let mut cursor = Cursor::new(&mut self.buffer);
        let mut streams = SmlStream::deserialize(&mut cursor)?;

        return match streams.pop() {
            Some(x) => SmlFile::deserialize(x),
            None => Err("No data received".to_string()),
        };
    }

    pub fn extract_produced_consumed(
        file: SmlFile,
    ) -> Result<(f64, f64), String> {
        let message = match file.messages.iter().find(|elem| {
            if let SmlBody::GetListResponse(_) = elem.body {
                return true;
            }
            return false;
        }) {
            Some(x) => x,
            None => {
                return Err("No SmlGetListResponse found in SmlFile".to_string())
            }
        };

        let values = match &message.body {
            SmlBody::GetListResponse(x) => &x.values,
            _ => {
                return Err(
                    "No SmlGetListResponse found in SmlMessage".to_string()
                )
            }
        };

        return values.iter().try_fold((0.0, 0.0), |mut acc, val| {
            if val.obj_name == SmlClient::OBIS_CONSUMED {
                acc.0 = Self::val_from_obis(&val)?;
            } else if val.obj_name == SmlClient::OBIS_PRODUCED {
                acc.1 = Self::val_from_obis(&val)?;
            }
            Ok(acc)
        });
    }

    fn val_from_obis(val: &doc::SmlListEntry) -> Result<f64, String> {
        if let Some(unit) = val.unit {
            if unit != SmlClient::SML_UNIT_WH {
                return Err(format!(
                    "Expected SML Unit 0x1E, found {:X}",
                    unit
                ));
            }
        }

        return match val.as_f64() {
            Some(x) => Ok(x),
            None => return Err("Could not convert SML data to number".into()),
        };
    }

    pub async fn get_consumed_produced(
        &mut self,
    ) -> Result<(f64, f64), String> {
        let data = self.get_sml_file().await?;
        return SmlClient::extract_produced_consumed(data);
    }
}
