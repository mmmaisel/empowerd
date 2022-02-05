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
use super::{parse_socketaddr_with_default, PollResult, PollState, Sources};
use crate::interval_sleep;
use crate::models::{InfluxObject, InfluxResult, SimpleMeter};
use chrono::{DateTime, Utc};
use slog::{error, trace, Logger};
use std::time::{Duration, UNIX_EPOCH};
use sunspec_client::SunspecClient;
use tokio::sync::watch;

pub struct SunspecSolarSource {
    canceled: watch::Receiver<PollState>,
    influx: influxdb::Client,
    name: String,
    interval: Duration,
    logger: Logger,
    client: SunspecClient,
}

impl SunspecSolarSource {
    pub fn new(
        canceled: watch::Receiver<PollState>,
        influx: influxdb::Client,
        name: String,
        interval: Duration,
        address: String,
        id: Option<u8>,
        logger: Logger,
    ) -> Result<Self, String> {
        let address = parse_socketaddr_with_default(&address, 502)?;
        let client = SunspecClient::new(address, id, Some(logger.clone()));

        return Ok(Self {
            canceled: canceled,
            influx: influx,
            name: name,
            interval: interval,
            logger: logger,
            client: client,
        });
    }

    pub async fn poll(&mut self) -> PollResult {
        let now = interval_sleep!(self);

        let mut context = match self.client.open().await {
            Ok(x) => x,
            Err(e) => {
                error!(self.logger, "Could not open sunspec connection: {}", e);
                return PollResult::Running;
            }
        };

        if self.client.models().is_empty() {
            if let Err(e) = self.client.introspect(&mut context).await {
                error!(
                    self.logger,
                    "Could not introspect sunspec device: {}", e
                );
                return PollResult::Running;
            }
        }

        let energy = match self.client.get_total_yield(&mut context).await {
            Ok(x) => x,
            Err(e) => {
                error!(self.logger, "Could not read energy yield: {}", e);
                return PollResult::Running;
            }
        };
        trace!(self.logger, "Total energy is {}", &energy);

        let power = match SimpleMeter::into_single(
            self.influx
                .json_query(SimpleMeter::query_last(&self.name))
                .await,
        ) {
            InfluxResult::Some(last_record) => {
                trace!(self.logger, "Read {:?} from database", last_record);
                3600.0 * (energy - last_record.energy)
                    / ((now - last_record.time.timestamp() as u64) as f64)
            }
            InfluxResult::None => 0.0,
            InfluxResult::Err(e) => {
                error!(
                    self.logger,
                    "Query {} database failed: {}", &self.name, e
                );
                return PollResult::Running;
            }
        };

        let record = SimpleMeter::new(
            DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_secs(now)),
            energy,
            power,
        );
        trace!(self.logger, "Writing {:?} to database", &record);
        if let Err(e) = self.influx.query(&record.save_query(&self.name)).await
        {
            error!(self.logger, "Save simple meter data failed, {}", e);
        }
        return PollResult::Running;
    }
}
