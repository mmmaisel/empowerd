use super::{Miner, MinerResult, MinerState};
use crate::models::{Battery, InfluxObject, InfluxResult};
use battery_client::BatteryClient;
use chrono::{DateTime, Utc};
use influxdb::InfluxDbWriteable;
use slog::{error, trace, Logger};
use std::time::{Duration, UNIX_EPOCH};
use tokio::sync::watch;

pub struct BatteryMiner {
    canceled: watch::Receiver<MinerState>,
    influx: influxdb::Client,
    interval: Duration,
    logger: Logger,
    battery_client: BatteryClient,
}

impl BatteryMiner {
    pub fn new(
        canceled: watch::Receiver<MinerState>,
        influx: influxdb::Client,
        interval: Duration,
        battery_addr: String,
        logger: Logger,
    ) -> Result<BatteryMiner, String> {
        let battery_client =
            BatteryClient::new(battery_addr, 502, Some(logger.clone()))?;
        return Ok(BatteryMiner {
            canceled: canceled,
            influx: influx,
            interval: interval,
            logger: logger,
            battery_client: battery_client,
        });
    }

    // TODO: dedup
    pub async fn mine(&mut self) -> MinerResult {
        let now = match Miner::sleep_aligned(
            self.interval,
            &mut self.canceled,
            &self.logger,
        )
        .await
        {
            Err(e) => {
                return MinerResult::Err(format!(
                    "sleep_aligned failed in BatteryMiner: {}",
                    e
                ));
            }
            Ok(state) => match state {
                MinerState::Canceled => return MinerResult::Canceled,
                MinerState::Running(x) => x,
            },
        };

        let (wh_in, wh_out, charge) =
            match self.battery_client.get_in_out_charge().await {
                Ok((x, y, z)) => (x as f64, y as f64, z),
                Err(e) => {
                    error!(self.logger, "Get battery data failed: {}", e);
                    return MinerResult::Running;
                }
            };

        let power = match Battery::into_single(
            self.influx.json_query(Battery::query_last()).await,
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
            wh_in,
            wh_out,
            charge.into(),
            power,
        );

        trace!(self.logger, "Writing {:?} to database", &battery);
        // XXX: extract save method
        if let Err(e) = self.influx.query(&battery.into_query("battery")).await
        {
            error!(self.logger, "Save BatteryData failed, {}", e);
        }
        return MinerResult::Running;
    }
}
