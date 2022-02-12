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
use std::time::{Duration, SystemTime};
use tokio::sync::watch;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub enum PollResult {
    Running,
    Canceled,
    Err(String),
}

#[derive(Debug)]
pub enum PollState {
    Running(u64),
    Canceled,
}

// TODO: add dummy source from file for testing
mod bresser6in1;
mod dachs_msr_s;
mod dummy;
mod ke_contact;
mod sml_meter;
mod sunny_boy_speedwire;
mod sunny_storage;
mod sunspec_solar;

pub struct Sources {
    logger: Logger,
    sources: FuturesUnordered<JoinHandle<PollResult>>,
    cancel: watch::Sender<PollState>,
}

macro_rules! polling_task {
    ($source:expr) => {
        tokio::task::spawn(async move {
            loop {
                let result = $source.poll().await;
                if let PollResult::Running = result {
                    continue;
                }
                return result;
            }
        })
    };
}

#[macro_export]
macro_rules! interval_sleep {
    ($self:expr) => {
        match Sources::sleep_aligned(
            $self.interval,
            &mut $self.canceled,
            &$self.logger,
            &$self.name,
        )
        .await
        {
            Err(e) => {
                return PollResult::Err(format!(
                    "sleep_aligned failed in {}:{}: {}",
                    std::any::type_name::<Self>(),
                    &$self.name,
                    e
                ));
            }
            Ok(state) => match state {
                PollState::Canceled => return PollResult::Canceled,
                PollState::Running(x) => x,
            },
        }
    };
}

impl Sources {
    pub fn new(logger: Logger, settings: &Settings) -> Result<Self, String> {
        let sources = FuturesUnordered::new();
        let (tx, rx) = watch::channel(PollState::Running(0));

        let influx_client = influxdb::Client::new(
            format!("http://{}", &settings.database.url),
            &settings.database.name,
        )
        .with_auth(&settings.database.user, &settings.database.password);

        for source in &settings.sources {
            match source {
                Source::SunnyIsland(settings) => {
                    let mut battery = sunny_storage::SunnyStorageSource::new(
                        rx.clone(),
                        influx_client.clone(),
                        settings.name.clone(),
                        "sunny_island",
                        Duration::from_secs(settings.poll_interval),
                        settings.address.clone(),
                        logger.clone(),
                    )?;
                    sources.push(polling_task!(battery));
                }
                Source::SunnyBoyStorage(settings) => {
                    let mut battery = sunny_storage::SunnyStorageSource::new(
                        rx.clone(),
                        influx_client.clone(),
                        settings.name.clone(),
                        "sunny_boy_storage",
                        Duration::from_secs(settings.poll_interval),
                        settings.address.clone(),
                        logger.clone(),
                    )?;
                    sources.push(polling_task!(battery));
                }
                Source::SunspecSolar(settings) => {
                    let mut sunspec = sunspec_solar::SunspecSolarSource::new(
                        rx.clone(),
                        influx_client.clone(),
                        settings.name.clone(),
                        Duration::from_secs(settings.poll_interval),
                        settings.address.clone(),
                        settings.modbus_id,
                        logger.clone(),
                    )?;
                    sources.push(polling_task!(sunspec));
                }
                Source::DachsMsrS(settings) => {
                    let mut dachs = dachs_msr_s::DachsMsrSSource::new(
                        rx.clone(),
                        influx_client.clone(),
                        settings.name.clone(),
                        Duration::from_secs(settings.poll_interval),
                        settings.address.clone(),
                        settings.password.clone(),
                        logger.clone(),
                    )?;
                    sources.push(polling_task!(dachs));
                }
                Source::KeContact(settings) => {
                    let mut kecontact = ke_contact::KeContactSource::new(
                        rx.clone(),
                        influx_client.clone(),
                        settings.name.clone(),
                        Duration::from_secs(settings.poll_interval),
                        settings.address.clone(),
                        logger.clone(),
                    )?;
                    sources.push(polling_task!(kecontact));
                }
                Source::SmlMeter(settings) => {
                    let mut meter = sml_meter::SmlMeterSource::new(
                        rx.clone(),
                        influx_client.clone(),
                        settings.name.clone(),
                        Duration::from_secs(settings.poll_interval),
                        settings.device.clone(),
                        settings.baud,
                        logger.clone(),
                    )?;
                    sources.push(polling_task!(meter));
                }
                Source::SunnyBoySpeedwire(settings) => {
                    let mut solar =
                        sunny_boy_speedwire::SunnyBoySpeedwireSource::new(
                            rx.clone(),
                            influx_client.clone(),
                            settings.name.clone(),
                            Duration::from_secs(settings.poll_interval),
                            settings.password.clone(),
                            settings.address.clone(),
                            logger.clone(),
                        )?;
                    sources.push(polling_task!(solar));
                }
                Source::Bresser6in1(settings) => {
                    let mut weather = bresser6in1::Bresser6in1Source::new(
                        rx.clone(),
                        influx_client.clone(),
                        settings.name.clone(),
                        Duration::from_secs(settings.poll_interval),
                        logger.clone(),
                    )?;
                    sources.push(polling_task!(weather));
                }
            }
        }

        if sources.is_empty() {
            warn!(logger, "No sources enabled, using dummy");
            let mut dummy = dummy::DummySource::new(
                rx,
                "dummy".into(),
                Duration::from_secs(86400),
                logger.clone(),
            )?;
            sources.push(polling_task!(dummy));
        }

        return Ok(Sources {
            logger,
            sources,
            cancel: tx,
        });
    }

    pub async fn run(&mut self) -> Result<(), ()> {
        while let Some(join_result) = self.sources.next().await {
            let result = match join_result {
                Ok(x) => x,
                Err(e) => {
                    error!(self.logger, "Join polling task failed: {}", e);
                    return Err(());
                }
            };

            match result {
                PollResult::Running => {}
                PollResult::Canceled => {
                    // XXX: print task name here
                    debug!(self.logger, "Task was canceled");
                }
                PollResult::Err(e) => {
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
        return match self.cancel.send(PollState::Canceled) {
            Ok(_) => {
                info!(self.logger, "Source polling canceled");
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
        canceled: &mut watch::Receiver<PollState>,
        logger: &Logger,
        ty: &str,
    ) -> Result<PollState, String> {
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
                return Ok(PollState::Canceled);
            }
            _ = tokio::time::sleep(sleep_time) => {
                trace!(logger, "wokeup");
                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map_err(|e| {
                        format!("System time is {:?} seconds before UNIX epoch", e)
                    })?.as_secs();
                return Ok(PollState::Running(now));
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
        Sources::sleep_duration(300, 1621753443)
    );
    assert_eq!(
        Duration::from_secs(30),
        Sources::sleep_duration(60, 1621754070)
    );
}
