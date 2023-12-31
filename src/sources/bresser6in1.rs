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
use super::SourceBase;
use crate::{models::Weather, task_group::TaskResult};
use bresser6in1_usb::{Client as BresserClient, PID, VID};
use slog::{debug, error, warn};
use std::time::Duration;

pub struct Bresser6in1Source {
    base: SourceBase,
    //bresser_client: BresserClient,
}

impl Bresser6in1Source {
    pub fn new(base: SourceBase) -> Self {
        Self { base }
    }

    pub async fn run(&mut self) -> TaskResult {
        let timing = match self.base.sleep_aligned().await {
            Ok(x) => x,
            Err(e) => return e,
        };

        let mut conn = match self.base.database.get().await {
            Ok(x) => x,
            Err(e) => {
                error!(
                    self.base.logger,
                    "Getting database connection from pool failed: {}", e
                );
                return TaskResult::Running;
            }
        };

        let logger2 = self.base.logger.clone();
        let mut weather_data = match tokio::task::block_in_place(move || {
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
        }) {
            Ok(x) => x,
            Err(e) => {
                error!(
                    self.base.logger,
                    "Get weather data failed, {}, giving up!", e
                );
                return TaskResult::Running;
            }
        };
        weather_data.timestamp = timing.now as u32;

        let record = Weather::new(weather_data);
        self.base.notify_processors(&record);
        if let Err(e) = record.insert(&mut conn, self.base.series_id).await {
            error!(
                self.base.logger,
                "Inserting {} record into database failed: {}",
                &self.base.name,
                e
            );
        }

        TaskResult::Running
    }
}
