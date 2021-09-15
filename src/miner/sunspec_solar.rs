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
use super::{parse_socketaddr_with_default, Miner, MinerResult, MinerState};
use crate::miner_sleep;
use crate::models::{InfluxObject, InfluxResult, Solar};
use chrono::{DateTime, Utc};
use slog::{error, trace, Logger};
use std::time::{Duration, UNIX_EPOCH};
use sunspec_client::SunspecClient;
use tokio::sync::watch;

pub struct SunspecSolarMiner {
    canceled: watch::Receiver<MinerState>,
    influx: influxdb::Client,
    name: String,
    interval: Duration,
    logger: Logger,
    client: SunspecClient,
}

impl SunspecSolarMiner {
    pub fn new(
        canceled: watch::Receiver<MinerState>,
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

    pub async fn mine(&mut self) -> MinerResult {
        let now = miner_sleep!(self);

        let mut context = match self.client.open().await {
            Ok(x) => x,
            Err(e) => {
                error!(self.logger, "Could not open sunspec connection: {}", e);
                return MinerResult::Running;
            }
        };

        if self.client.models().is_empty() {
            if let Err(e) = self.client.introspect(&mut context).await {
                error!(
                    self.logger,
                    "Could not introspect sunspec device: {}", e
                );
                return MinerResult::Running;
            }
        }

        let energy = match self.client.get_total_yield(&mut context).await {
            Ok(x) => x,
            Err(e) => {
                error!(self.logger, "Could not read energy yield: {}", e);
                return MinerResult::Running;
            }
        };
        trace!(self.logger, "Total energy is {}", &energy);

        let power = match Solar::into_single(
            self.influx.json_query(Solar::query_last(&self.name)).await,
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
                return MinerResult::Running;
            }
        };

        let solar = Solar::new(
            DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_secs(now)),
            energy,
            power,
        );
        trace!(self.logger, "Writing {:?} to database", &solar);
        if let Err(e) = self.influx.query(&solar.save_query(&self.name)).await {
            error!(self.logger, "Save SolarData failed, {}", e);
        }
        return MinerResult::Running;
    }
}
