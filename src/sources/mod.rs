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
use crate::task_group::{TaskGroup, TaskState};
use crate::task_loop;
use futures::stream::FuturesUnordered;
use slog::{debug, trace, warn, Logger};
use std::time::{Duration, SystemTime};
use tokio::sync::watch;

// TODO: add dummy source from file for testing
mod bresser6in1;
mod dachs_msr_s;
mod dummy;
mod ke_contact;
mod sml_meter;
mod sunny_boy_speedwire;
mod sunny_storage;
mod sunspec_solar;

#[macro_export]
macro_rules! interval_sleep {
    ($self:expr) => {
        match crate::sources::sleep_aligned(
            $self.interval,
            &mut $self.canceled,
            &$self.logger,
            &$self.name,
        )
        .await
        {
            Err(e) => {
                return crate::task_group::TaskResult::Err(format!(
                    "sleep_aligned failed in {}:{}: {}",
                    std::any::type_name::<Self>(),
                    &$self.name,
                    e
                ));
            }
            Ok(state) => match state {
                crate::task_group::TaskState::Canceled => {
                    return crate::task_group::TaskResult::Canceled(
                        $self.name.clone(),
                    )
                }
                crate::task_group::TaskState::Running(x) => x,
            },
        }
    };
}

pub fn new(logger: Logger, settings: &Settings) -> Result<TaskGroup, String> {
    let sources = FuturesUnordered::new();
    let (tx, rx) = watch::channel(TaskState::Running(0));

    let influx_client = influxdb::Client::new(
        format!("http://{}", &settings.database.url),
        &settings.database.name,
    )
    .with_auth(&settings.database.user, &settings.database.password);

    // TODO: this should belong to main/toplevel once connections to
    //   processors and sinks are made.
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
                sources.push(task_loop!(battery));
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
                sources.push(task_loop!(battery));
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
                sources.push(task_loop!(sunspec));
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
                sources.push(task_loop!(dachs));
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
                sources.push(task_loop!(kecontact));
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
                sources.push(task_loop!(meter));
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
                sources.push(task_loop!(solar));
            }
            Source::Bresser6in1(settings) => {
                let mut weather = bresser6in1::Bresser6in1Source::new(
                    rx.clone(),
                    influx_client.clone(),
                    settings.name.clone(),
                    Duration::from_secs(settings.poll_interval),
                    logger.clone(),
                )?;
                sources.push(task_loop!(weather));
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
        sources.push(task_loop!(dummy));
    }

    Ok(TaskGroup::new("sources".into(), sources, logger, tx))
}

fn sleep_duration(interval: u64, now: u64) -> Duration {
    return Duration::from_secs(interval - (now % interval));
}

// TODO: use a watch channel to notify processors of new source values
pub async fn sleep_aligned(
    interval: Duration,
    canceled: &mut watch::Receiver<TaskState>,
    logger: &Logger,
    ty: &str,
) -> Result<TaskState, String> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| {
            format!("System time is {:?} seconds before UNIX epoch", e)
        })?;

    let interval_s = interval.as_secs();
    let sleep_time = sleep_duration(interval_s, now.as_secs());
    debug!(logger, "{}: sleep until {:?}", ty, sleep_time);
    tokio::select! {
        _ = canceled.changed() => {
            trace!(logger, "sleep was canceled");
            return Ok(TaskState::Canceled);
        }
        _ = tokio::time::sleep(sleep_time) => {
            trace!(logger, "wokeup");
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map_err(|e| {
                    format!("System time is {:?} seconds before UNIX epoch", e)
                })?.as_secs();
            return Ok(TaskState::Running(now));
        }
        else => {
            return Err("sleep_aligned returned".into());
        }
    }
}

#[test]
fn test_sleep_duration() {
    assert_eq!(Duration::from_secs(57), sleep_duration(300, 1621753443));
    assert_eq!(Duration::from_secs(30), sleep_duration(60, 1621754070));
}
