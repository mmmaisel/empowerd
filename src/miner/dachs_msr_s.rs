use super::{Miner, MinerResult, MinerState};
use crate::miner_sleep;
use crate::models::{Dachs, InfluxObject, InfluxResult};
use chrono::{DateTime, Utc};
use dachs_client::DachsClient;
use slog::{error, trace, Logger};
use std::time::{Duration, UNIX_EPOCH};
use tokio::sync::watch;

pub struct DachsMsrSMiner {
    canceled: watch::Receiver<MinerState>,
    influx: influxdb::Client,
    name: String,
    interval: Duration,
    logger: Logger,
    dachs_client: DachsClient,
}

impl DachsMsrSMiner {
    pub fn new(
        canceled: watch::Receiver<MinerState>,
        influx: influxdb::Client,
        name: String,
        interval: Duration,
        dachs_addr: String,
        dachs_pw: String,
        logger: Logger,
    ) -> Result<DachsMsrSMiner, String> {
        return Ok(DachsMsrSMiner {
            canceled: canceled,
            influx: influx,
            name: name,
            interval: interval,
            logger: logger.clone(),
            dachs_client: DachsClient::new(dachs_addr, dachs_pw, Some(logger)),
        });
    }

    pub async fn mine(&mut self) -> MinerResult {
        let now = miner_sleep!(self);

        let dachs_runtime = match self.dachs_client.get_runtime().await {
            Ok(runtime) => {
                trace!(self.logger, "Runtime: {} s", runtime);
                runtime
            }
            Err(err) => {
                error!(self.logger, "{}", err);
                return MinerResult::Running;
            }
        };
        let dachs_energy = match self.dachs_client.get_total_energy().await {
            Ok(energy) => {
                trace!(self.logger, "Energy: {} kWh", energy);
                energy
            }
            Err(err) => {
                error!(self.logger, "{}", err);
                return MinerResult::Running;
            }
        };

        let power = match Dachs::into_single(
            self.influx.json_query(Dachs::query_last(&self.name)).await,
        ) {
            InfluxResult::Some(last_record) => {
                // TODO: derive nonlinear power from delta timestamp and delta runtime
                if dachs_runtime != last_record.runtime as i32 {
                    800.0
                } else {
                    0.0
                }
            }
            InfluxResult::None => 0.0,
            InfluxResult::Err(e) => {
                error!(self.logger, "Query dachs database failed: {}", e);
                return MinerResult::Running;
            }
        };

        let dachs = Dachs::new(
            DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_secs(now)),
            dachs_energy.into(),
            power,
            dachs_runtime.into(),
        );

        trace!(self.logger, "Writing {:?} to database", &dachs);
        if let Err(e) = self.influx.query(&dachs.save_query(&self.name)).await {
            error!(self.logger, "Save DachsData failed, {}", e);
        }
        return MinerResult::Running;
    }
}
