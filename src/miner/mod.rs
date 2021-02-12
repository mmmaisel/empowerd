use crate::Settings;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use slog::{debug, error, info, Logger};
use std::time::{Duration, SystemTime};
use tokio::task::JoinHandle;

type MinerResult = Result<String, String>;

mod battery;
mod dachs;
mod meter;
mod solar;
mod weather;

pub struct Miner {
    logger: Logger,
    miners: FuturesUnordered<JoinHandle<MinerResult>>,
}

impl Miner {
    pub fn new(logger: Logger, settings: &Settings) -> Result<Miner, String> {
        let tasks = FuturesUnordered::new();

        let battery = battery::BatteryMiner::new(
            Duration::from_secs(settings.battery_poll_interval),
            logger.clone(),
        )?;
        let dachs = dachs::DachsMiner::new(
            Duration::from_secs(settings.dachs_poll_interval),
            logger.clone(),
        )?;
        let meter = meter::MeterMiner::new(
            Duration::from_secs(settings.meter_poll_interval),
            logger.clone(),
        )?;
        let solar = solar::SolarMiner::new(
            Duration::from_secs(settings.sma_poll_interval),
            logger.clone(),
        )?;
        let weather = weather::WeatherMiner::new(
            Duration::from_secs(settings.weather_poll_interval),
            logger.clone(),
        )?;
        tasks.push(tokio::task::spawn(async move { battery.mine().await }));
        tasks.push(tokio::task::spawn(async move { dachs.mine().await }));
        tasks.push(tokio::task::spawn(async move { meter.mine().await }));
        tasks.push(tokio::task::spawn(async move { solar.mine().await }));
        tasks.push(tokio::task::spawn(async move { weather.mine().await }));

        return Ok(Miner {
            logger: logger,
            miners: tasks,
        });
    }

    pub async fn run(&mut self) -> bool {
        while let Some(t) = self.miners.next().await {
            let t = t.unwrap();
            if let Err(e) = t {
                error!(self.logger, "Task failed: {:?}", &e);
                return false;
            } else {
                debug!(self.logger, "Task completed: {:?}", &t.unwrap());
            }
        }
        return true;
    }

    pub async fn sleep_aligned(
        interval: Duration,
        logger: &Logger,
    ) -> Result<(), String> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| {
                format!("System time is {:?} seconds before UNIX epoch", e)
            })?;

        let interval_s = interval.as_secs();
        let sleep =
            Duration::from_secs(interval_s - (now.as_secs() % interval_s));
        debug!(logger, "sleep by: {:?}", sleep);
        // TODO: use sleep until?
        //        tokio::time::sleep(sleep).await;
        return Ok(());
    }
}
