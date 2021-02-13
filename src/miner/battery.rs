use super::{Miner, MinerResult, MinerState};
use slog::{debug, error, info, Logger};
use std::time::Duration;
use tokio::sync::watch;

pub struct BatteryMiner {
    logger: Logger,
    canceled: watch::Receiver<MinerState>,
    //influx_conn: Client,
    interval: Duration,
    //battery_client: BatteryClient,
}

impl BatteryMiner {
    pub fn new(
        interval: Duration,
        canceled: watch::Receiver<MinerState>,
        logger: Logger,
    ) -> Result<BatteryMiner, String> {
        return Ok(BatteryMiner {
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
                    "sleep_aligned failed in BatteryMiner: {}",
                    e
                ));
            }
            Ok(state) => {
                if let MinerState::Canceled = state {
                    return MinerResult::Canceled;
                }
            }
        }
        return MinerResult::Err("battery not implemented yet".into());
    }
}
