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
use crate::settings::{Settings, SourceType};
use crate::task_group::{TaskGroup, TaskGroupBuilder, TaskResult, TaskState};
use crate::task_loop;
use slog::{debug, error, trace, Logger};
use std::collections::BTreeMap;
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

pub use bresser6in1::Bresser6in1Source;
pub use dachs_msr_s::DachsMsrSSource;
pub use debug::DebugSource;
pub use dummy::DummySource;
pub use ke_contact::KeContactSource;
pub use sml_meter::SmlMeterSource;
pub use sunny_boy_speedwire::SunnyBoySpeedwireSource;
pub use sunny_storage::SunnyStorageSource;
pub use sunspec_solar::SunspecSolarSource;

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

    pub fn add_processor(
        &mut self,
        name: &str,
        settings: &Settings,
        outputs: &mut BTreeMap<String, watch::Receiver<Model>>,
    ) -> &mut Self {
        if settings.has_processor(name) {
            let (tx, rx) = watch::channel(Model::None);
            self.processors = Some(tx);
            outputs.insert(name.into(), rx);
        }
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
            processors.send_replace(record.clone().into());
        }
    }
}

pub fn polling_tasks(
    logger: Logger,
    settings: &Settings,
) -> Result<(TaskGroup, BTreeMap<String, watch::Receiver<Model>>), String> {
    let tasks = TaskGroupBuilder::new("sources".into(), logger.clone());
    let mut outputs = BTreeMap::<String, watch::Receiver<Model>>::new();
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

        match &source.variant {
            SourceType::Debug(setting) => {
                base_builder
                    .name(source.name.clone())
                    .interval(Duration::from_secs(setting.poll_interval))
                    .add_processor(&source.name, settings, &mut outputs);
                let mut source = DebugSource::new(base_builder.build());
                tasks.add_task(task_loop!(source));
            }
            SourceType::SunnyIsland(setting) => {
                base_builder
                    .name(source.name.clone())
                    .interval(Duration::from_secs(setting.poll_interval))
                    .add_processor(&source.name, settings, &mut outputs);
                let mut source = SunnyStorageSource::new(
                    base_builder.build(),
                    "sunny_island",
                    setting.address.clone(),
                )?;
                tasks.add_task(task_loop!(source));
            }
            SourceType::SunnyBoyStorage(setting) => {
                base_builder
                    .name(source.name.clone())
                    .interval(Duration::from_secs(setting.poll_interval))
                    .add_processor(&source.name, settings, &mut outputs);
                let mut source = SunnyStorageSource::new(
                    base_builder.build(),
                    "sunny_boy_storage",
                    setting.address.clone(),
                )?;
                tasks.add_task(task_loop!(source));
            }
            SourceType::SunspecSolar(setting) => {
                base_builder
                    .name(source.name.clone())
                    .interval(Duration::from_secs(setting.poll_interval))
                    .add_processor(&source.name, settings, &mut outputs);
                let mut source = SunspecSolarSource::new(
                    base_builder.build(),
                    setting.address.clone(),
                    setting.modbus_id,
                )?;
                tasks.add_task(task_loop!(source));
            }
            SourceType::DachsMsrS(setting) => {
                base_builder
                    .name(source.name.clone())
                    .interval(Duration::from_secs(setting.poll_interval))
                    .add_processor(&source.name, settings, &mut outputs);
                let mut source = DachsMsrSSource::new(
                    base_builder.build(),
                    setting.address.clone(),
                    setting.password.clone(),
                );
                tasks.add_task(task_loop!(source));
            }
            SourceType::KeContact(setting) => {
                base_builder
                    .name(source.name.clone())
                    .interval(Duration::from_secs(setting.poll_interval))
                    .add_processor(&source.name, settings, &mut outputs);
                let mut source = KeContactSource::new(
                    base_builder.build(),
                    setting.address.clone(),
                )?;
                tasks.add_task(task_loop!(source));
            }
            SourceType::SmlMeter(setting) => {
                base_builder
                    .name(source.name.clone())
                    .interval(Duration::from_secs(setting.poll_interval))
                    .add_processor(&source.name, settings, &mut outputs);
                let mut source = SmlMeterSource::new(
                    base_builder.build(),
                    setting.device.clone(),
                    setting.baud,
                )?;
                tasks.add_task(task_loop!(source));
            }
            SourceType::SunnyBoySpeedwire(setting) => {
                base_builder
                    .name(source.name.clone())
                    .interval(Duration::from_secs(setting.poll_interval))
                    .add_processor(&source.name, settings, &mut outputs);
                let mut source = SunnyBoySpeedwireSource::new(
                    base_builder.build(),
                    setting.password.clone(),
                    setting.address.clone(),
                )?;
                tasks.add_task(task_loop!(source));
            }
            SourceType::Bresser6in1(setting) => {
                base_builder
                    .name(source.name.clone())
                    .interval(Duration::from_secs(setting.poll_interval))
                    .add_processor(&source.name, settings, &mut outputs);
                let mut source = Bresser6in1Source::new(base_builder.build());
                tasks.add_task(task_loop!(source));
            }
        }
    }

    if !tasks.has_tasks() {
        debug!(logger, "No sources enabled, using dummy");
        let mut dummy = DummySource::new(SourceBase::new(
            "dummy".into(),
            Duration::from_secs(86400),
            influx_client,
            tasks.cancel_rx(),
            logger,
            None,
        ));
        tasks.add_task(task_loop!(dummy));
    }

    Ok((tasks.build(), outputs))
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
