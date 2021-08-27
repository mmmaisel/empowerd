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
use super::{Miner, MinerResult, MinerState};
use crate::miner_sleep;
use crate::models::{Battery, InfluxObject, InfluxResult};
use chrono::{DateTime, Utc};
use slog::{error, trace, Logger};
use std::net::SocketAddr;
use std::time::{Duration, UNIX_EPOCH};
use sunny_storage_client::{
    SunnyBoyStorageClient, SunnyIslandClient, SunnyStorageClient,
};
use tokio::sync::watch;

pub struct SunnyStorageMiner {
    canceled: watch::Receiver<MinerState>,
    influx: influxdb::Client,
    name: String,
    interval: Duration,
    logger: Logger,
    battery_client: Box<dyn SunnyStorageClient + Send + Sync>,
}

impl SunnyStorageMiner {
    pub fn new(
        canceled: watch::Receiver<MinerState>,
        influx: influxdb::Client,
        name: String,
        r#type: &'static str,
        interval: Duration,
        battery_addr: String,
        logger: Logger,
    ) -> Result<Self, String> {
        let address: SocketAddr = match battery_addr.parse() {
            Ok(x) => x,
            Err(_) => match format!("{}:502", battery_addr).parse() {
                Ok(x) => x,
                Err(e) => return Err(e.to_string()),
            },
        };
        let battery_client: Box<dyn SunnyStorageClient + Send + Sync> =
            match r#type {
                "sunny_island" => Box::new(SunnyIslandClient::new(
                    address,
                    Some(logger.clone()),
                )?),
                "sunny_boy_storage" => Box::new(SunnyBoyStorageClient::new(
                    address,
                    Some(logger.clone()),
                )?),
                _ => {
                    return Err(format!(
                        "ArgumentError: SunnyStorageClient type {} is invalid.",
                        r#type
                    ))
                }
            };

        return Ok(Self {
            canceled: canceled,
            influx: influx,
            name: name,
            interval: interval,
            logger: logger,
            battery_client: battery_client,
        });
    }

    pub async fn mine(&mut self) -> MinerResult {
        let now = miner_sleep!(self);

        let (wh_in, wh_out, charge) =
            match self.battery_client.get_in_out_charge().await {
                Ok((x, y, z)) => (x as f64, y as f64, z),
                Err(e) => {
                    error!(self.logger, "Get battery data failed: {}", e);
                    return MinerResult::Running;
                }
            };

        let power = match Battery::into_single(
            self.influx
                .json_query(Battery::query_last(&self.name))
                .await,
        ) {
            InfluxResult::Some(last_record) => {
                3600.0
                    * (wh_in
                        - last_record.energy_in
                        - (wh_out - last_record.energy_out))
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

        let battery = Battery::new(
            DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_secs(now)),
            charge,
            wh_in,
            wh_out,
            power,
        );

        trace!(self.logger, "Writing {:?} to database", &battery);
        if let Err(e) = self.influx.query(&battery.save_query(&self.name)).await
        {
            error!(self.logger, "Save BatteryData failed, {}", e);
        }
        return MinerResult::Running;
    }
}
