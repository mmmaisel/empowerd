use super::Miner;
use slog::{debug, error, info, Logger};
use std::time::Duration;

pub struct DachsMiner {
    logger: Logger,
    //influx_conn: Client,
    interval: Duration,
    //dachs_client: DachsClient,
}

impl DachsMiner {
    pub fn new(
        interval: Duration,
        logger: Logger,
    ) -> Result<DachsMiner, String> {
        return Ok(DachsMiner {
            logger: logger,
            interval: interval,
        });
    }

    pub async fn mine(&self) -> Result<String, String> {
        if let Err(e) = Miner::sleep_aligned(self.interval, &self.logger).await
        {
            return Err(format!("sleep_aligned failed in DachsMiner: {}", e));
        }
        return Err("dachs not implemented yet".into());
    }
}
