use super::{Miner, MinerResult, MinerState};
use slog::{debug, error, info, Logger};
use std::time::Duration;
use tokio::sync::watch;

pub struct DachsMiner {
    logger: Logger,
    canceled: watch::Receiver<MinerState>,
    //influx_conn: Client,
    interval: Duration,
    //dachs_client: DachsClient,
}

impl DachsMiner {
    pub fn new(
        interval: Duration,
        canceled: watch::Receiver<MinerState>,
        logger: Logger,
    ) -> Result<DachsMiner, String> {
        return Ok(DachsMiner {
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
                    "sleep_aligned failed in DachsMiner: {}",
                    e
                ));
            }
            Ok(state) => {
                if let MinerState::Canceled = state {
                    return MinerResult::Canceled;
                }
            }
        }
        return MinerResult::Err("dachs not implemented yet".into());
    }
}
