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
use crate::models::{InfluxResult, SimpleMeter};
use crate::task_group::TaskResult;
use chrono::{DateTime, Utc};
use slog::{error, trace};
use std::time::{Duration, UNIX_EPOCH};
use sunspec_client::SunspecClient;

pub struct SunspecSolarSource {
    base: SourceBase,
    client: SunspecClient,
}

impl SunspecSolarSource {
    pub fn new(
        base: SourceBase,
        address: String,
        id: Option<u8>,
    ) -> Result<Self, String> {
        let address = parse_socketaddr_with_default(&address, 502)?;
        let client = SunspecClient::new(address, id, Some(base.logger.clone()));

        Ok(Self { base, client })
    }

    pub async fn run(&mut self) -> TaskResult {
        let now = match self.base.sleep_aligned().await {
            Ok(x) => x,
            Err(e) => return e,
        };

        let mut context = match self.client.open().await {
            Ok(x) => x,
            Err(e) => {
                error!(
                    self.base.logger,
                    "Could not open sunspec connection: {}", e
                );
                return TaskResult::Running;
            }
        };

        if self.client.models().is_empty() {
            if let Err(e) = self.client.introspect(&mut context).await {
                error!(
                    self.base.logger,
                    "Could not introspect sunspec device: {}", e
                );
                return TaskResult::Running;
            }
        }

        let energy = match self.client.get_total_yield(&mut context).await {
            Ok(x) => x,
            Err(e) => {
                error!(self.base.logger, "Could not read energy yield: {}", e);
                return TaskResult::Running;
            }
        };
        trace!(self.base.logger, "Total energy is {}", &energy);

        let power = match self.base.query_last::<SimpleMeter>().await {
            InfluxResult::Some(last_record) => {
                trace!(
                    self.base.logger,
                    "Read {:?} from database",
                    last_record
                );
                3600.0 * (energy - last_record.energy)
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

        let record = SimpleMeter::new(
            DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_secs(now)),
            energy,
            power,
        );
        self.base.notify_processors(&record);
        let _: Result<(), ()> = self.base.save_record(record).await;
        TaskResult::Running
    }
}
