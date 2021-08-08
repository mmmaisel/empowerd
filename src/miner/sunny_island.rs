use super::{Miner, MinerResult, MinerState};
use crate::miner_sleep;
use crate::models::{Battery, InfluxObject, InfluxResult};
use chrono::{DateTime, Utc};
use slog::{error, trace, Logger};
use std::time::{Duration, UNIX_EPOCH};
use sunny_island_client::SunnyIslandClient;
use tokio::sync::watch;

pub struct SunnyIslandMiner {
    canceled: watch::Receiver<MinerState>,
    influx: influxdb::Client,
    name: String,
    interval: Duration,
    logger: Logger,
    battery_client: SunnyIslandClient,
}

impl SunnyIslandMiner {
    pub fn new(
        canceled: watch::Receiver<MinerState>,
        influx: influxdb::Client,
        name: String,
        interval: Duration,
        battery_addr: String,
        logger: Logger,
    ) -> Result<SunnyIslandMiner, String> {
        let battery_client =
            SunnyIslandClient::new(battery_addr, 502, Some(logger.clone()))?;
        return Ok(SunnyIslandMiner {
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
                error!(self.logger, "Query battery database failed: {}", e);
                return MinerResult::Running;
            }
        };

        let battery = Battery::new(
            DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_secs(now)),
            (charge as f64) / 100.0,
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
