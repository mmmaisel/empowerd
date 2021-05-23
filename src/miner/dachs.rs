use super::{Miner, MinerResult, MinerState};
use crate::models::{Dachs, InfluxObject, InfluxResult};
use chrono::{DateTime, Utc};
use dachs_client::DachsClient;
use slog::{error, trace, Logger};
use std::time::{Duration, UNIX_EPOCH};
use tokio::sync::watch;

pub struct DachsMiner {
    canceled: watch::Receiver<MinerState>,
    influx: influxdb::Client,
    interval: Duration,
    logger: Logger,
    dachs_client: DachsClient,
}

impl DachsMiner {
    pub fn new(
        canceled: watch::Receiver<MinerState>,
        influx: influxdb::Client,
        interval: Duration,
        dachs_addr: String,
        dachs_pw: String,
        logger: Logger,
    ) -> Result<DachsMiner, String> {
        return Ok(DachsMiner {
            canceled: canceled,
            influx: influx,
            interval: interval,
            logger: logger.clone(),
            dachs_client: DachsClient::new(dachs_addr, dachs_pw, Some(logger)),
        });
    }

    // TODO: dedup
    pub async fn mine(&mut self) -> MinerResult {
        let now = match Miner::sleep_aligned(
            self.interval,
            &mut self.canceled,
            &self.logger,
            "Dachs",
        )
        .await
        {
            Err(e) => {
                return MinerResult::Err(format!(
                    "sleep_aligned failed in DachsMiner: {}",
                    e
                ));
            }
            Ok(state) => match state {
                MinerState::Canceled => return MinerResult::Canceled,
                MinerState::Running(x) => x,
            },
        };

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
            self.influx.json_query(Dachs::query_last()).await,
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
        if let Err(e) = self.influx.query(&dachs.save_query()).await {
            error!(self.logger, "Save DachsData failed, {}", e);
        }
        return MinerResult::Running;
    }
}
