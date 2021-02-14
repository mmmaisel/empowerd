use super::{Miner, MinerResult, MinerState};
use slog::{debug, error, info, Logger};
use std::time::Duration;
use tokio::sync::watch;

pub struct SolarMiner {
    canceled: watch::Receiver<MinerState>,
    influx: influxdb::Client,
    interval: Duration,
    logger: Logger,
    //sma_client: SmaClient,
    //sma_pw: String,
    //sma_addr: net::SocketAddr,
}

impl SolarMiner {
    pub fn new(
        canceled: watch::Receiver<MinerState>,
        influx: influxdb::Client,
        interval: Duration,
        logger: Logger,
    ) -> Result<SolarMiner, String> {
        return Ok(SolarMiner {
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
                    "sleep_aligned failed in SolarMiner: {}",
                    e
                ));
            }
            Ok(state) => {
                if let MinerState::Canceled = state {
                    return MinerResult::Canceled;
                }
            }
        }
        return MinerResult::Err("solar not implemented yet".into());
    }
}
