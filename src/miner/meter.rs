use super::{Miner, MinerResult, MinerState};
use slog::{debug, error, info, Logger};
use std::time::Duration;
use tokio::sync::watch;

pub struct MeterMiner {
    logger: Logger,
    canceled: watch::Receiver<MinerState>,
    //influx_conn: Client,
    interval: Duration,
    //sml_client: SmlClient,
}

impl MeterMiner {
    pub fn new(
        interval: Duration,
        canceled: watch::Receiver<MinerState>,
        logger: Logger,
    ) -> Result<MeterMiner, String> {
        return Ok(MeterMiner {
            logger: logger,
            canceled: canceled,
            interval: interval,
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
                    "sleep_aligned failed in MeterMiner: {}",
                    e
                ));
            }
            Ok(state) => {
                if let MinerState::Canceled = state {
                    return MinerResult::Canceled;
                }
            }
        }
        return MinerResult::Err("meter not implemented yet".into());
    }
}
