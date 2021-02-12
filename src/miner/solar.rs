use super::Miner;
use slog::{debug, error, info, Logger};
use std::time::Duration;

pub struct SolarMiner {
    logger: Logger,
    //influx_conn: Client,
    interval: Duration,
    //sma_client: SmaClient,
    //sma_pw: String,
    //sma_addr: net::SocketAddr,
}

impl SolarMiner {
    pub fn new(
        interval: Duration,
        logger: Logger,
    ) -> Result<SolarMiner, String> {
        return Ok(SolarMiner {
            logger: logger,
            interval: interval,
        });
    }

    pub async fn mine(&self) -> Result<String, String> {
        if let Err(e) = Miner::sleep_aligned(self.interval, &self.logger).await
        {
            return Err(format!("sleep_aligned failed in SolarMiner: {}", e));
        }
        return Err("solar not implemented yet".into());
    }
}
