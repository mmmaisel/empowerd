use crate::Settings;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use slog::{debug, error, info, trace, Logger};
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
mod solar_sma;
mod weather;

pub struct Miner {
    logger: Logger,
    miners: FuturesUnordered<JoinHandle<MinerResult>>,
    cancel: watch::Sender<MinerState>,
}

macro_rules! miner_task {
    ($miner:expr) => {
        tokio::task::spawn(async move {
            loop {
                let result = $miner.mine().await;
                if let MinerResult::Running = result {
                    continue;
                }
                return result;
            }
        })
    };
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

        if settings.enable_battery {
            let mut battery = battery::BatteryMiner::new(
                rx.clone(),
                influx_client.clone(),
                Duration::from_secs(settings.battery_poll_interval),
                settings.battery_addr.clone(),
                logger.clone(),
            )?;
            miners.push(miner_task!(battery));
        }

        if settings.enable_dachs {
            let mut dachs = dachs::DachsMiner::new(
                rx.clone(),
                influx_client.clone(),
                Duration::from_secs(settings.dachs_poll_interval),
                settings.dachs_addr.clone(),
                settings.dachs_pw.clone(),
                logger.clone(),
            )?;
            miners.push(miner_task!(dachs));
        }

        if settings.enable_meter {
            let mut meter = meter::MeterMiner::new(
                rx.clone(),
                influx_client.clone(),
                Duration::from_secs(settings.meter_poll_interval),
                settings.meter_device.clone(),
                settings.meter_baud,
                logger.clone(),
            )?;
            miners.push(miner_task!(meter));
        }

        if settings.enable_solar {
            if settings.solar_type == "SMA" {
                let mut solar = solar_sma::SolarMiner::new(
                    rx.clone(),
                    influx_client.clone(),
                    Duration::from_secs(settings.solar_poll_interval),
                    settings.solar_pw.clone(),
                    settings.solar_addr.clone(),
                    logger.clone(),
                )?;
                miners.push(miner_task!(solar));
            } else {
                return Err(format!(
                    "Found unknown solar type '{}'",
                    settings.solar_type
                ));
            }
        }

        if settings.enable_weather {
            let mut weather = weather::WeatherMiner::new(
                rx.clone(),
                influx_client.clone(),
                Duration::from_secs(settings.weather_poll_interval),
                logger.clone(),
            )?;
            miners.push(miner_task!(weather));
        }

        if miners.is_empty() {
            return Err("No miners are enabled".into());
        }

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
                    // XXX: print task name here
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
        if self.cancel.is_closed() {
            return Ok(());
        }
        return match self.cancel.send(MinerState::Canceled) {
            Ok(_) => {
                info!(self.logger, "Miner canceled");
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        };
    }

    fn sleep_duration(interval: u64, now: u64) -> Duration {
        return Duration::from_secs(interval - (now % interval));
    }

    pub async fn sleep_aligned(
        interval: Duration,
        canceled: &mut watch::Receiver<MinerState>,
        logger: &Logger,
        ty: &str,
    ) -> Result<MinerState, String> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| {
                format!("System time is {:?} seconds before UNIX epoch", e)
            })?;

        let interval_s = interval.as_secs();
        let sleep_time = Self::sleep_duration(interval_s, now.as_secs());
        debug!(logger, "{}: sleep until {:?}", ty, sleep_time);
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

#[test]
fn test_sleep_duration() {
    assert_eq!(
        Duration::from_secs(57),
        Miner::sleep_duration(300, 1621753443)
    );
    assert_eq!(
        Duration::from_secs(30),
        Miner::sleep_duration(60, 1621754070)
    );
}
