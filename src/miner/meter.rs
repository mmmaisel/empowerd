use super::Miner;
use slog::{debug, error, info, Logger};
use std::time::Duration;

pub struct MeterMiner {
    logger: Logger,
    //influx_conn: Client,
    interval: Duration,
    //sml_client: SmlClient,
}

impl MeterMiner {
    pub fn new(
        interval: Duration,
        logger: Logger,
    ) -> Result<MeterMiner, String> {
        return Ok(MeterMiner {
            logger: logger,
            interval: interval,
        });
    }

    pub async fn mine(&self) -> Result<String, String> {
        if let Err(e) = Miner::sleep_aligned(self.interval, &self.logger).await
        {
            return Err(format!("sleep_aligned failed in MeterMiner: {}", e));
        }
        return Err("meter not implemented yet".into());
    }
}
