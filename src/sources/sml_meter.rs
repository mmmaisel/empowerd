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
use crate::{
    models::{
        units::{second, watt, watt_hour, Energy, Power, Time},
        BidirMeter,
    },
    task_group::TaskResult,
    Error,
};
use slog::{debug, error, trace, warn, Logger};
use sml_client::SmlClient;
use std::time::Duration;

pub struct SmlMeterSource {
    base: SourceBase,
    sml_client: SmlClient,
    meter_device: String,
}

impl SmlMeterSource {
    pub fn new(
        base: SourceBase,
        meter_device: String,
        meter_baud: u32,
    ) -> Result<Self, String> {
        if base.interval < Duration::from_secs(5) {
            return Err("SmlMeterSource:poll_interval must be >= 5".into());
        }
        let logger = base.logger.clone();
        Ok(Self {
            base,
            sml_client: SmlClient::new(
                meter_device.clone(),
                meter_baud,
                Some(logger),
            ),
            meter_device,
        })
    }

    pub fn logger(&self) -> &Logger {
        &self.base.logger
    }

    pub async fn run(&mut self) -> TaskResult {
        let timing = self.base.sleep_aligned().await?;
        let mut conn = self.base.database.get().await.map_err(|e| {
            Error::Temporary(format!(
                "Getting database connection from pool failed: {e}",
            ))
        })?;

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
            Ok((x, y)) => {
                (Energy::new::<watt_hour>(x), Energy::new::<watt_hour>(y))
            }
            Err(e) => {
                return Err(Error::Temporary(format!(
                    "Get electric meter data failed, {e}, giving up!",
                )))
            }
        };

        let mut record = BidirMeter {
            time: Time::new::<second>(timing.now as f64),
            energy_in: consumed,
            energy_out: produced,
            power: Power::new::<watt>(0.0),
        };
        record.power =
            match BidirMeter::last(&mut conn, self.base.series_id).await {
                Ok(last_record) => {
                    trace!(
                        self.base.logger,
                        "Read {:?} from database",
                        last_record
                    );
                    record.calc_power(&last_record)
                }
                Err(Error::NotFound) => Power::new::<watt>(0.0),
                Err(e) => {
                    return Err(Error::Temporary(format!(
                        "Query {} database failed: {}",
                        &self.base.name, e,
                    )))
                }
            };

        self.base.notify_processors(&record);
        record
            .insert(&mut conn, self.base.series_id)
            .await
            .map_err(|e| {
                Error::Temporary(format!(
                    "Inserting {} record into database failed: {}",
                    &self.base.name, e,
                ))
            })?;

        Ok(())
    }
}
