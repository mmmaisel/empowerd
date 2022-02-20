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
use crate::models::{InfluxObject, InfluxResult, Model};
use crate::settings::{Settings, Source};
use crate::task_group::{TaskGroup, TaskGroupBuilder, TaskResult, TaskState};
use crate::task_loop;
use slog::{debug, error, trace, warn, Logger};
use std::time::{Duration, SystemTime};
use tokio::sync::watch;

mod bresser6in1;
mod dachs_msr_s;
mod debug;
mod dummy;
mod ke_contact;
mod sml_meter;
mod sunny_boy_speedwire;
mod sunny_storage;
mod sunspec_solar;

#[derive(Debug)]
pub struct SourceBaseBuilder {
    name: String,
    interval: Duration,
    influx: influxdb::Client,
    canceled: watch::Receiver<TaskState>,
    logger: Logger,
    processors: Option<watch::Sender<Model>>,
}

impl SourceBaseBuilder {
    pub fn new(
        influx: influxdb::Client,
        canceled: watch::Receiver<TaskState>,
        logger: Logger,
    ) -> Self {
        Self {
            name: String::new(),
            interval: Duration::new(0, 0),
            influx,
            canceled,
            logger,
            processors: None,
        }
    }

    pub fn name(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }

    pub fn interval(&mut self, interval: Duration) -> &mut Self {
        self.interval = interval;
        self
    }

    pub fn processors(
        &mut self,
        processors: watch::Sender<Model>,
    ) -> &mut Self {
        self.processors = Some(processors);
        self
    }

    pub fn build(self) -> SourceBase {
        SourceBase::new(
            self.name,
            self.interval,
            self.influx,
            self.canceled,
            self.logger,
            self.processors,
        )
    }
}

pub struct SourceBase {
    name: String,
    interval: Duration,
    influx: influxdb::Client,
    canceled: watch::Receiver<TaskState>,
    logger: Logger,
    processors: Option<watch::Sender<Model>>,
}

impl SourceBase {
    pub fn new(
        name: String,
        interval: Duration,
        influx: influxdb::Client,
        canceled: watch::Receiver<TaskState>,
        logger: Logger,
        processors: Option<watch::Sender<Model>>,
    ) -> Self {
        Self {
            canceled,
            influx,
            name,
            interval,
            logger,
            processors,
        }
    }

    pub async fn sleep_aligned(&mut self) -> Result<u64, TaskResult> {
        match sleep_aligned(
            self.interval,
            &mut self.canceled,
            &self.logger,
            &self.name,
        )
        .await
        {
            Err(e) => Err(TaskResult::Err(format!(
                "sleep_aligned failed in {}:{}: {}",
                std::any::type_name::<Self>(),
                &self.name,
                e
            ))),
            Ok(state) => match state {
                TaskState::Canceled => {
                    Err(TaskResult::Canceled(self.name.clone()))
                }
                TaskState::Running(x) => Ok(x),
            },
        }
    }

    pub async fn save_record<U, T>(&self, record: T) -> Result<(), ()>
    where
        U: 'static + Send + for<'de> serde::Deserialize<'de>,
        T: InfluxObject<U> + std::fmt::Debug,
    {
        trace!(self.logger, "Writing {:?} to database", &record);
        if let Err(e) = self.influx.query(&record.save_query(&self.name)).await
        {
            error!(
                self.logger,
                "Save {} data failed, {}",
                std::any::type_name::<T>(),
                e
            );
            return Err(());
        }
        Ok(())
    }

    pub async fn query_last<T>(&self) -> InfluxResult<T>
    where
        T: 'static + Send + for<'de> serde::Deserialize<'de> + InfluxObject<T>,
    {
        T::into_single(self.influx.json_query(T::query_last(&self.name)).await)
    }

    pub fn notify_processors<T>(&self, record: &T)
    where
        T: Clone + Into<Model>,
    {
        if let Some(processors) = &self.processors {
            if let Err(e) = processors.send(record.clone().into()) {
                error!(
                    self.logger,
                    "Notifying processors from {:} failed: {}",
                    &self.name,
                    e.to_string()
                )
            }
        }
    }
}

pub fn new(logger: Logger, settings: &Settings) -> Result<TaskGroup, String> {
    let tasks = TaskGroupBuilder::new("sources".into(), logger.clone());
    let influx_client = influxdb::Client::new(
        format!("http://{}", &settings.database.url),
        &settings.database.name,
    )
    .with_auth(&settings.database.user, &settings.database.password);

    for source in &settings.sources {
        let mut base_builder = SourceBaseBuilder::new(
            influx_client.clone(),
            tasks.cancel_rx(),
            logger.clone(),
        );

        match source {
            Source::Debug(settings) => {
                base_builder
                    .name(settings.name.clone())
                    .interval(Duration::from_secs(settings.poll_interval));
                let mut debug = debug::DebugSource::new(base_builder.build());
                tasks.add_task(task_loop!(debug));
            }
            Source::SunnyIsland(settings) => {
                base_builder
                    .name(settings.name.clone())
                    .interval(Duration::from_secs(settings.poll_interval));
                let mut battery = sunny_storage::SunnyStorageSource::new(
                    base_builder.build(),
                    "sunny_island",
                    settings.address.clone(),
                )?;
                tasks.add_task(task_loop!(battery));
            }
            Source::SunnyBoyStorage(settings) => {
                base_builder
                    .name(settings.name.clone())
                    .interval(Duration::from_secs(settings.poll_interval));
                let mut battery = sunny_storage::SunnyStorageSource::new(
                    base_builder.build(),
                    "sunny_boy_storage",
                    settings.address.clone(),
                )?;
                tasks.add_task(task_loop!(battery));
            }
            Source::SunspecSolar(settings) => {
                base_builder
                    .name(settings.name.clone())
                    .interval(Duration::from_secs(settings.poll_interval));
                let mut sunspec = sunspec_solar::SunspecSolarSource::new(
                    base_builder.build(),
                    settings.address.clone(),
                    settings.modbus_id,
                )?;
                tasks.add_task(task_loop!(sunspec));
            }
            Source::DachsMsrS(settings) => {
                base_builder
                    .name(settings.name.clone())
                    .interval(Duration::from_secs(settings.poll_interval));
                let mut dachs = dachs_msr_s::DachsMsrSSource::new(
                    base_builder.build(),
                    settings.address.clone(),
                    settings.password.clone(),
                );
                tasks.add_task(task_loop!(dachs));
            }
            Source::KeContact(settings) => {
                base_builder
                    .name(settings.name.clone())
                    .interval(Duration::from_secs(settings.poll_interval));
                let mut kecontact = ke_contact::KeContactSource::new(
                    base_builder.build(),
                    settings.address.clone(),
                )?;
                tasks.add_task(task_loop!(kecontact));
            }
            Source::SmlMeter(settings) => {
                base_builder
                    .name(settings.name.clone())
                    .interval(Duration::from_secs(settings.poll_interval));
                let mut meter = sml_meter::SmlMeterSource::new(
                    base_builder.build(),
                    settings.device.clone(),
                    settings.baud,
                )?;
                tasks.add_task(task_loop!(meter));
            }
            Source::SunnyBoySpeedwire(settings) => {
                base_builder
                    .name(settings.name.clone())
                    .interval(Duration::from_secs(settings.poll_interval));
                let mut solar =
                    sunny_boy_speedwire::SunnyBoySpeedwireSource::new(
                        base_builder.build(),
                        settings.password.clone(),
                        settings.address.clone(),
                    )?;
                tasks.add_task(task_loop!(solar));
            }
            Source::Bresser6in1(settings) => {
                base_builder
                    .name(settings.name.clone())
                    .interval(Duration::from_secs(settings.poll_interval));
                let mut weather =
                    bresser6in1::Bresser6in1Source::new(base_builder.build());
                tasks.add_task(task_loop!(weather));
            }
        }
    }

    if !tasks.has_tasks() {
        warn!(logger, "No sources enabled, using dummy");
        let mut dummy = dummy::DummySource::new(SourceBase::new(
            "dummy".into(),
            Duration::from_secs(86400),
            influx_client,
            tasks.cancel_rx(),
            logger,
            None,
        ));
        tasks.add_task(task_loop!(dummy));
    }

    Ok(tasks.build())
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
