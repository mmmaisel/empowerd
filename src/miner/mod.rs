/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2022 Max Maisel

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
\******************************************************************************/
use crate::settings::{Settings, Source};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use slog::{debug, error, info, trace, warn, Logger};
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

mod bresser6in1;
mod dachs_msr_s;
mod dummy;
mod kecontact;
mod sml_meter;
mod sunny_boy_speedwire;
mod sunny_storage;
mod sunspec_solar;

pub fn parse_socketaddr_with_default(
    addr: &str,
    default_port: u16,
) -> Result<SocketAddr, String> {
    match addr.parse() {
        Ok(x) => Ok(x),
        Err(_) => match format!("{}:{}", addr, default_port).parse() {
            Ok(x) => Ok(x),
            Err(e) => return Err(e.to_string()),
        },
    }
}

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

#[macro_export]
macro_rules! miner_sleep {
    ($self:expr) => {
        match Miner::sleep_aligned(
            $self.interval,
            &mut $self.canceled,
            &$self.logger,
            &$self.name,
        )
        .await
        {
            Err(e) => {
                return MinerResult::Err(format!(
                    "sleep_aligned failed in {}:{}: {}",
                    std::any::type_name::<Self>(),
                    &$self.name,
                    e
                ));
            }
            Ok(state) => match state {
                MinerState::Canceled => return MinerResult::Canceled,
                MinerState::Running(x) => x,
            },
        }
    };
}

impl Miner {
    pub fn new(logger: Logger, settings: &Settings) -> Result<Miner, String> {
        let miners = FuturesUnordered::new();
        let (tx, rx) = watch::channel(MinerState::Running(0));

        let influx_client = influxdb::Client::new(
            format!("http://{}", &settings.database.url),
            &settings.database.name,
        )
        .with_auth(&settings.database.user, &settings.database.password);

        for source in &settings.sources {
            match source {
                Source::SunnyIsland(settings) => {
                    let mut battery = sunny_storage::SunnyStorageMiner::new(
                        rx.clone(),
                        influx_client.clone(),
                        settings.name.clone(),
                        "sunny_island",
                        Duration::from_secs(settings.poll_interval),
                        settings.address.clone(),
                        logger.clone(),
                    )?;
                    miners.push(miner_task!(battery));
                }
                Source::SunnyBoyStorage(settings) => {
                    let mut battery = sunny_storage::SunnyStorageMiner::new(
                        rx.clone(),
                        influx_client.clone(),
                        settings.name.clone(),
                        "sunny_boy_storage",
                        Duration::from_secs(settings.poll_interval),
                        settings.address.clone(),
                        logger.clone(),
                    )?;
                    miners.push(miner_task!(battery));
                }
                Source::SunspecSolar(settings) => {
                    let mut sunspec = sunspec_solar::SunspecSolarMiner::new(
                        rx.clone(),
                        influx_client.clone(),
                        settings.name.clone(),
                        Duration::from_secs(settings.poll_interval),
                        settings.address.clone(),
                        settings.modbus_id,
                        logger.clone(),
                    )?;
                    miners.push(miner_task!(sunspec));
                }
                Source::DachsMsrS(settings) => {
                    let mut dachs = dachs_msr_s::DachsMsrSMiner::new(
                        rx.clone(),
                        influx_client.clone(),
                        settings.name.clone(),
                        Duration::from_secs(settings.poll_interval),
                        settings.address.clone(),
                        settings.password.clone(),
                        logger.clone(),
                    )?;
                    miners.push(miner_task!(dachs));
                }
                Source::KeContact(settings) => {
                    let mut kecontact = kecontact::KeContactMiner::new(
                        rx.clone(),
                        influx_client.clone(),
                        settings.name.clone(),
                        Duration::from_secs(settings.poll_interval),
                        settings.address.clone(),
                        logger.clone(),
                    )?;
                    miners.push(miner_task!(kecontact));
                }
                Source::SmlMeter(settings) => {
                    let mut meter = sml_meter::SmlMeterMiner::new(
                        rx.clone(),
                        influx_client.clone(),
                        settings.name.clone(),
                        Duration::from_secs(settings.poll_interval),
                        settings.device.clone(),
                        settings.baud,
                        logger.clone(),
                    )?;
                    miners.push(miner_task!(meter));
                }
                Source::SunnyBoySpeedwire(settings) => {
                    let mut solar =
                        sunny_boy_speedwire::SunnyBoySpeedwireMiner::new(
                            rx.clone(),
                            influx_client.clone(),
                            settings.name.clone(),
                            Duration::from_secs(settings.poll_interval),
                            settings.password.clone(),
                            settings.address.clone(),
                            logger.clone(),
                        )?;
                    miners.push(miner_task!(solar));
                }
                Source::Bresser6in1(settings) => {
                    let mut weather = bresser6in1::Bresser6in1Miner::new(
                        rx.clone(),
                        influx_client.clone(),
                        settings.name.clone(),
                        Duration::from_secs(settings.poll_interval),
                        logger.clone(),
                    )?;
                    miners.push(miner_task!(weather));
                }
            }
        }

        if miners.is_empty() {
            warn!(logger, "No miners enabled, using dummy");
            let mut dummy = dummy::DummyMiner::new(
                rx,
                "dummy".into(),
                Duration::from_secs(86400),
                logger.clone(),
            )?;
            miners.push(miner_task!(dummy));
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
