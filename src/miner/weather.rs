use super::Miner;
use slog::{debug, error, info, Logger};
use std::time::Duration;

pub struct WeatherMiner {
    logger: Logger,
    //influx_conn: Client,
    interval: Duration,
    //bresser_client: BresserClient,
}

impl WeatherMiner {
    pub fn new(
        interval: Duration,
        logger: Logger,
    ) -> Result<WeatherMiner, String> {
        return Ok(WeatherMiner {
            logger: logger,
            interval: interval,
        });
    }

    pub async fn mine(&self) -> Result<String, String> {
        if let Err(e) = Miner::sleep_aligned(self.interval, &self.logger).await
        {
            return Err(format!("sleep_aligned failed in WeatherMiner: {}", e));
        }
        return Err("weather not implemented yet".into());
    }
}
