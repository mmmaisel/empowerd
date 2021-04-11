use crate::Settings;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use slog::{debug, error, info, trace, Logger};
use std::net::SocketAddr;
use std::time::{Duration, SystemTime};
use tokio::sync::watch;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub enum MinerResult {
    Running,
    Canceled,
    Err(String),
}

#[derive(Debug)]
pub enum MinerState {
    Running(u64),
    Canceled,
}

mod battery;
mod dachs;
mod meter;
mod solar;
mod weather;

pub struct Miner {
    logger: Logger,
    miners: FuturesUnordered<JoinHandle<MinerResult>>,
    cancel: watch::Sender<MinerState>,
}

impl Miner {
    pub fn new(logger: Logger, settings: &Settings) -> Result<Miner, String> {
        let miners = FuturesUnordered::new();
        let (tx, rx) = watch::channel(MinerState::Running(0));

        let influx_client = influxdb::Client::new(
            format!("http://{}", &settings.db_url),
            &settings.db_name,
        )
        .with_auth(&settings.db_user, &settings.db_pw);

        let sma_socket_addr: SocketAddr = match settings.sma_addr.parse() {
            Ok(x) => x,
            Err(e) => return Err(format!("Could not parse sma_addr: {}", e)),
        };

        let mut battery = battery::BatteryMiner::new(
            rx.clone(),
            influx_client.clone(),
            Duration::from_secs(settings.battery_poll_interval),
            logger.clone(),
        )?;
        let mut dachs = dachs::DachsMiner::new(
            rx.clone(),
            influx_client.clone(),
            Duration::from_secs(settings.dachs_poll_interval),
            logger.clone(),
        )?;
        let mut meter = meter::MeterMiner::new(
            rx.clone(),
            influx_client.clone(),
            Duration::from_secs(settings.meter_poll_interval),
            logger.clone(),
        )?;
        let mut solar = solar::SolarMiner::new(
            rx.clone(),
            influx_client.clone(),
            Duration::from_secs(settings.sma_poll_interval),
            settings.sma_pw.clone(),
            sma_socket_addr,
            logger.clone(),
        )?;
        let mut weather = weather::WeatherMiner::new(
            rx.clone(),
            influx_client.clone(),
            Duration::from_secs(settings.weather_poll_interval),
            logger.clone(),
        )?;

        miners.push(tokio::task::spawn(async move { battery.mine().await }));
        miners.push(tokio::task::spawn(async move { dachs.mine().await }));
        miners.push(tokio::task::spawn(async move { meter.mine().await }));
        miners.push(tokio::task::spawn(async move { solar.mine().await }));
        miners.push(tokio::task::spawn(async move { weather.mine().await }));

        return Ok(Miner {
            logger: logger,
            miners: miners,
            cancel: tx,
        });
    }

    pub async fn run(&mut self) -> Result<(), ()> {
        while let Some(join_result) = self.miners.next().await {
            let result = match join_result {
                Ok(x) => x,
                Err(e) => {
                    error!(self.logger, "Joining miner task failed: {}", e);
                    return Err(());
                }
            };

            match result {
                MinerResult::Running => {}
                MinerResult::Canceled => {
                    debug!(self.logger, "Task was canceled");
                }
                MinerResult::Err(e) => {
                    error!(self.logger, "Task failed: {:?}", e);
                    return Err(());
                }
            }
        }
        return Ok(());
    }

    pub fn cancel(&mut self) -> Result<(), String> {
        return match self.cancel.send(MinerState::Canceled) {
            Ok(x) => {
                info!(self.logger, "Miner canceled");
                Ok(x)
            }
            Err(e) => Err(e.to_string()),
        };
    }

    pub async fn sleep_aligned(
        interval: Duration,
        canceled: &mut watch::Receiver<MinerState>,
        logger: &Logger,
    ) -> Result<MinerState, String> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| {
                format!("System time is {:?} seconds before UNIX epoch", e)
            })?;

        let interval_s = interval.as_secs();
        let sleep_time =
            Duration::from_secs(interval_s * (now.as_secs() % interval_s));
        debug!(logger, "sleep until {:?}", sleep_time);
        tokio::select! {
            _ = canceled.changed() => {
                trace!(logger, "sleep was canceled");
                return Ok(MinerState::Canceled);
            }
            _ = tokio::time::sleep(sleep_time) => {
                trace!(logger, "wokeup");
                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map_err(|e| {
                        format!("System time is {:?} seconds before UNIX epoch", e)
                    })?.as_secs();
                return Ok(MinerState::Running(now));
            }
            else => {
                return Err("sleep_aligned returned".into());
            }
        }
    }
}
