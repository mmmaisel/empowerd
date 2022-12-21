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
use crate::misc::parse_socketaddr_with_default;
use crate::models::{Battery, InfluxResult};
use crate::task_group::TaskResult;
use chrono::{DateTime, Utc};
use slog::error;
use std::time::{Duration, UNIX_EPOCH};
use sunny_storage_client::{
    SunnyBoyStorageClient, SunnyIslandClient, SunnyStorageClient,
};

pub struct SunnyStorageSource {
    base: SourceBase,
    battery_client: Box<dyn SunnyStorageClient + Send + Sync>,
}

impl SunnyStorageSource {
    pub fn new(
        base: SourceBase,
        r#type: &'static str,
        address: String,
    ) -> Result<Self, String> {
        let address = parse_socketaddr_with_default(&address, 502)?;
        let battery_client: Box<dyn SunnyStorageClient + Send + Sync> =
            match r#type {
                "sunny_island" => Box::new(SunnyIslandClient::new(
                    address,
                    Some(base.logger.clone()),
                )?),
                "sunny_boy_storage" => Box::new(SunnyBoyStorageClient::new(
                    address,
                    Some(base.logger.clone()),
                )?),
                _ => {
                    return Err(format!(
                        "ArgumentError: SunnyStorageClient type {} is invalid.",
                        r#type
                    ))
                }
            };

        Ok(Self {
            base,
            battery_client: battery_client,
        })
    }

    pub async fn run(&mut self) -> TaskResult {
        let timing = match self.base.sleep_aligned().await {
            Ok(x) => x,
            Err(e) => return e,
        };

        let (wh_in, wh_out, charge) =
            match self.battery_client.get_in_out_charge().await {
                Ok((x, y, z)) => (x as f64, y as f64, z),
                Err(e) => {
                    error!(self.base.logger, "Get battery data failed: {}", e);
                    return TaskResult::Running;
                }
            };

        let power = match self.base.query_last::<Battery>().await {
            InfluxResult::Some(last_record) => {
                3600.0
                    * (wh_in
                        - last_record.energy_in
                        - (wh_out - last_record.energy_out))
                    / ((timing.now - last_record.time.timestamp() as u64)
                        as f64)
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

        let record = Battery::new(
            DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_secs(timing.now)),
            charge,
            wh_in,
            wh_out,
            power,
        );
        self.base.notify_processors(&record);
        let _: Result<(), ()> = self.base.save_record(record).await;
        TaskResult::Running
    }
}
