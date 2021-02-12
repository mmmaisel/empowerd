use super::Miner;
use slog::{debug, error, info, Logger};
use std::time::Duration;

pub struct BatteryMiner {
    logger: Logger,
    //influx_conn: Client,
    interval: Duration,
    //battery_client: BatteryClient,
}

impl BatteryMiner {
    pub fn new(
        interval: Duration,
        logger: Logger,
    ) -> Result<BatteryMiner, String> {
        return Ok(BatteryMiner {
            logger: logger,
            interval: interval,
        });
    }

    pub async fn mine(&self) -> Result<String, String> {
        if let Err(e) = Miner::sleep_aligned(self.interval, &self.logger).await
        {
            return Err(format!("sleep_aligned failed in BatteryMiner: {}", e));
        }
        return Err("battery not implemented yet".into());
    }
}
