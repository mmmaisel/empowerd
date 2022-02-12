/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2022 Max Maisel

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
use crate::interval_sleep;
use crate::models::{InfluxObject, Weather};
use crate::task_group::{TaskResult, TaskState};
use bresser6in1_usb::{Client as BresserClient, PID, VID};
use slog::{debug, error, trace, warn, Logger};
use std::time::Duration;
use tokio::sync::watch;

pub struct Bresser6in1Source {
    canceled: watch::Receiver<TaskState>,
    influx: influxdb::Client,
    name: String,
    interval: Duration,
    logger: Logger,
    //bresser_client: BresserClient,
}

impl Bresser6in1Source {
    pub fn new(
        canceled: watch::Receiver<TaskState>,
        influx: influxdb::Client,
        name: String,
        interval: Duration,
        logger: Logger,
    ) -> Result<Self, String> {
        return Ok(Self {
            canceled: canceled,
            influx: influx,
            name: name,
            interval: interval,
            logger: logger.clone(),
        });
    }

    pub async fn run(&mut self) -> TaskResult {
        let now = interval_sleep!(self);

        let logger2 = self.logger.clone();
        let weather_data = match tokio::task::spawn_blocking(move || {
            // TODO: move bresser client (allocated buffers, ...) back to Miner struct
            let mut bresser_client = BresserClient::new(Some(logger2.clone()));
            let mut weather_data = bresser_client.read_data();
            for i in 1..4u8 {
                if let Err(e) = weather_data {
                    if i == 2 {
                        match usb_reset::reset_vid_pid(VID, PID) {
                            Ok(()) => {
                                warn!(
                                    logger2,
                                    "Reset device {:X}:{:X}", VID, PID
                                );
                                std::thread::sleep(Duration::from_secs(5));
                            }
                            Err(e) => {
                                error!(
                                    logger2,
                                    "Reset device {:X}:{:X} failed: {}",
                                    VID,
                                    PID,
                                    e
                                );
                            }
                        }
                    }
                    debug!(
                        logger2,
                        "Get weather data failed, {}, retrying...", e
                    );
                    weather_data = bresser_client.read_data();
                } else {
                    break;
                }
            }
            return weather_data;
        })
        .await
        {
            Ok(x) => x,
            Err(e) => {
                error!(
                    self.logger,
                    "Joining blocking Bresser USB task failed: {}", e
                );
                return TaskResult::Running;
            }
        };
        let mut weather_data = match weather_data {
            Ok(x) => x,
            Err(e) => {
                error!(
                    self.logger,
                    "Get weather data failed, {}, giving up!", e
                );
                return TaskResult::Running;
            }
        };
        weather_data.timestamp = now as u32;

        let record = Weather::new(weather_data);
        trace!(self.logger, "Writing {:?} to database", &record);
        if let Err(e) = self.influx.query(&record.save_query(&self.name)).await
        {
            error!(self.logger, "Save weather data failed, {}", e);
        }
        return TaskResult::Running;
    }
}
