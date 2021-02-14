use super::{Miner, MinerResult, MinerState};
use slog::{debug, error, info, Logger};
use std::time::Duration;
use tokio::sync::watch;

pub struct WeatherMiner {
    canceled: watch::Receiver<MinerState>,
    influx: influxdb::Client,
    interval: Duration,
    logger: Logger,
    //bresser_client: BresserClient,
}

impl WeatherMiner {
    pub fn new(
        canceled: watch::Receiver<MinerState>,
        influx: influxdb::Client,
        interval: Duration,
        logger: Logger,
    ) -> Result<WeatherMiner, String> {
        return Ok(WeatherMiner {
            canceled: canceled,
            influx: influx,
            interval: interval,
            logger: logger,
        });
    }

    // TODO: dedup
    pub async fn mine(&mut self) -> MinerResult {
        match Miner::sleep_aligned(
            self.interval,
            &mut self.canceled,
            &self.logger,
        )
        .await
        {
            Err(e) => {
                return MinerResult::Err(format!(
                    "sleep_aligned failed in WeatherMiner: {}",
                    e
                ));
            }
            Ok(state) => {
                if let MinerState::Canceled = state {
                    return MinerResult::Canceled;
                }
            }
        }
        return MinerResult::Err("weather not implemented yet".into());
    }
}
