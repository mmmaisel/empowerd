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
use crate::models::{BidirectionalMeter, InfluxResult, Model};
use crate::task_group::{TaskResult, TaskState};
use chrono::{DateTime, Utc};
use slog::{debug, error, trace, warn, Logger};
use sml_client::SmlClient;
use std::time::{Duration, UNIX_EPOCH};
use tokio::sync::watch;

pub struct SmlMeterSource {
    base: SourceBase,
    sml_client: SmlClient,
    meter_device: String,
}

impl SmlMeterSource {
    pub fn new(
        canceled: watch::Receiver<TaskState>,
        influx: influxdb::Client,
        name: String,
        interval: Duration,
        meter_device: String,
        meter_baud: u32,
        logger: Logger,
        processors: Option<watch::Sender<Model>>,
    ) -> Result<Self, String> {
        if interval < Duration::from_secs(5) {
            return Err("SmlMeterSource:poll_interval must be >= 5".into());
        }
        Ok(Self {
            base: SourceBase::new(
                canceled,
                influx,
                name,
                interval,
                logger.clone(),
                processors,
            ),
            sml_client: SmlClient::new(
                meter_device.clone(),
                meter_baud,
                Some(logger),
            ),
            meter_device,
        })
    }

    pub async fn run(&mut self) -> TaskResult {
        let now = match self.base.sleep_aligned().await {
            Ok(x) => x,
            Err(e) => return e,
        };

        let mut meter_data = self.sml_client.get_consumed_produced().await;
        for i in 1..4u8 {
            if let Err(e) = meter_data {
                if i == 2 {
                    match usb_reset::reset_path(&self.meter_device) {
                        Ok(()) => {
                            warn!(
                                self.base.logger,
                                "Reset device {}", &self.meter_device
                            );
                            tokio::time::sleep(Duration::from_secs(5)).await;
                        }
                        Err(e) => {
                            error!(
                                self.base.logger,
                                "Reset device {} failed: {}",
                                &self.meter_device,
                                e
                            );
                        }
                    }
                }
                debug!(
                    self.base.logger,
                    "Get electric meter data failed, {}, retrying...", e
                );
                meter_data = self.sml_client.get_consumed_produced().await;
            } else {
                break;
            }
        }
        let (consumed, produced) = match meter_data {
            Ok((x, y)) => (x, y),
            Err(e) => {
                error!(
                    self.base.logger,
                    "Get electric meter data failed, {}, giving up!", e
                );
                return TaskResult::Running;
            }
        };

        let power = match self.base.query_last::<BidirectionalMeter>().await {
            InfluxResult::Some(last_record) => {
                trace!(
                    self.base.logger,
                    "Read {:?} from database",
                    last_record
                );
                3600.0
                    * (consumed
                        - last_record.energy_consumed
                        - (produced - last_record.energy_produced))
                    / ((now - last_record.time.timestamp() as u64) as f64)
            }
            InfluxResult::None => 0.0,
            InfluxResult::Err(e) => {
                error!(
                    self.base.logger,
                    "Query {} database failed: {}", &self.base.name, e
                );
                return TaskResult::Running;
            }
        };

        let record = BidirectionalMeter::new(
            DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_secs(now)),
            consumed,
            produced,
            power,
        );
        self.base.notify_processors(&record);
        let _: Result<(), ()> = self.base.save_record(record).await;
        TaskResult::Running
    }
}
