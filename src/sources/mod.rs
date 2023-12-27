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
use crate::task_group::{
    task_loop, TaskGroup, TaskGroupBuilder, TaskResult, TaskState, TaskTiming,
};
use slog::{debug, error, trace, Logger};
use std::collections::BTreeMap;
use std::time::{Duration, SystemTime};
use tokio::sync::watch;

mod bresser6in1;
mod dachs_msr_s;
mod debug;
mod dummy;
mod ke_contact;
mod lambda_heat_pump;
mod sma_meter;
mod sml_meter;
mod sunny_boy_speedwire;
mod sunny_storage;
mod sunspec_solar;

pub use bresser6in1::Bresser6in1Source;
pub use dachs_msr_s::DachsMsrSSource;
pub use debug::DebugSource;
pub use dummy::DummySource;
pub use ke_contact::KeContactSource;
pub use lambda_heat_pump::LambdaHeatPumpSource;
pub use sma_meter::SmaMeterSource;
pub use sml_meter::SmlMeterSource;
pub use sunny_boy_speedwire::SunnyBoySpeedwireSource;
pub use sunny_storage::SunnyStorageSource;
pub use sunspec_solar::SunspecSolarSource;

#[derive(Debug)]
pub struct SourceBaseBuilder {
    name: String,
    interval: Duration,
    oversample_factor: u64,
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
            oversample_factor: 1,
            influx,
            canceled,
            logger,
            processors: None,
        }
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    pub fn oversample_factor(mut self, oversample_factor: u64) -> Self {
        self.oversample_factor = oversample_factor;
        self
    }

    pub fn add_processor(
        mut self,
        name: &str,
        settings: &Settings,
        outputs: &mut BTreeMap<String, watch::Receiver<Model>>,
    ) -> Self {
        if settings.has_processor(name) {
            let (tx, rx) = watch::channel(Model::None);
            self.processors = Some(tx);
            outputs.insert(name.into(), rx);
        }
        self
    }

    pub fn build(self) -> SourceBase {
        SourceBase {
            name: self.name,
            interval: self.interval,
            oversample_factor: self.oversample_factor,
            influx: self.influx,
            canceled: self.canceled,
            logger: self.logger,
            processors: self.processors,
        }
    }
}

pub struct SourceBase {
    name: String,
    interval: Duration,
    oversample_factor: u64,
    influx: influxdb::Client,
    canceled: watch::Receiver<TaskState>,
    logger: Logger,
    processors: Option<watch::Sender<Model>>,
}

impl SourceBase {
    pub async fn sleep_aligned(&mut self) -> Result<TaskTiming, TaskResult> {
        match sleep_aligned(
            self.interval,
            self.oversample_factor,
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
        let base_builder = SourceBaseBuilder::new(
            influx_client.clone(),
            tasks.cancel_rx(),
            logger.clone(),
        );

        match &source.variant {
            SourceType::Debug(setting) => {
                let mut source = DebugSource::new(
                    base_builder
                        .name(source.name.clone())
                        .interval(Duration::from_secs(setting.poll_interval))
                        .add_processor(&source.name, settings, &mut outputs)
                        .build(),
                );
                tasks.add_task(task_loop!(source));
            }
            SourceType::SunnyIsland(setting) => {
                let mut source = SunnyStorageSource::new(
                    base_builder
                        .name(source.name.clone())
                        .interval(Duration::from_secs(setting.poll_interval))
                        .add_processor(&source.name, settings, &mut outputs)
                        .build(),
                    "sunny_island",
                    setting.address.clone(),
                )?;
                tasks.add_task(task_loop!(source));
            }
            SourceType::SunnyBoyStorage(setting) => {
                let mut source = SunnyStorageSource::new(
                    base_builder
                        .name(source.name.clone())
                        .interval(Duration::from_secs(setting.poll_interval))
                        .add_processor(&source.name, settings, &mut outputs)
                        .build(),
                    "sunny_boy_storage",
                    setting.address.clone(),
                )?;
                tasks.add_task(task_loop!(source));
            }
            SourceType::SunspecSolar(setting) => {
                let mut source = SunspecSolarSource::new(
                    base_builder
                        .name(source.name.clone())
                        .interval(Duration::from_secs(setting.poll_interval))
                        .add_processor(&source.name, settings, &mut outputs)
                        .build(),
                    setting.address.clone(),
                    setting.modbus_id,
                )?;
                tasks.add_task(task_loop!(source));
            }
            SourceType::DachsMsrS(setting) => {
                let mut source = DachsMsrSSource::new(
                    base_builder
                        .name(source.name.clone())
                        .interval(Duration::from_secs(setting.poll_interval))
                        .add_processor(&source.name, settings, &mut outputs)
                        .build(),
                    setting.address.clone(),
                    setting.password.clone(),
                );
                tasks.add_task(task_loop!(source));
            }
            SourceType::KeContact(setting) => {
                let mut source = KeContactSource::new(
                    base_builder
                        .name(source.name.clone())
                        .interval(Duration::from_secs(setting.poll_interval))
                        .add_processor(&source.name, settings, &mut outputs)
                        .build(),
                    setting.address.clone(),
                )?;
                tasks.add_task(task_loop!(source));
            }
            SourceType::LambdaHeatPump(setting) => {
                let mut source = LambdaHeatPumpSource::new(
                    base_builder
                        .name(source.name.clone())
                        .interval(Duration::from_secs(setting.poll_interval))
                        .oversample_factor(setting.oversample_factor)
                        .add_processor(&source.name, settings, &mut outputs)
                        .build(),
                    setting.address.clone(),
                )?;
                tasks.add_task(task_loop!(source));
            }
            SourceType::SmaMeter(setting) => {
                let mut source = SmaMeterSource::new(
                    base_builder
                        .name(source.name.clone())
                        .interval(Duration::from_secs(setting.poll_interval))
                        .add_processor(&source.name, settings, &mut outputs)
                        .build(),
                    setting.address.clone(),
                    setting.bind_address.clone(),
                )?;
                tasks.add_task(task_loop!(source));
            }
            SourceType::SmlMeter(setting) => {
                let mut source = SmlMeterSource::new(
                    base_builder
                        .name(source.name.clone())
                        .interval(Duration::from_secs(setting.poll_interval))
                        .add_processor(&source.name, settings, &mut outputs)
                        .build(),
                    setting.device.clone(),
                    setting.baud,
                )?;
                tasks.add_task(task_loop!(source));
            }
            SourceType::SunnyBoySpeedwire(setting) => {
                let mut source = SunnyBoySpeedwireSource::new(
                    base_builder
                        .name(source.name.clone())
                        .interval(Duration::from_secs(setting.poll_interval))
                        .add_processor(&source.name, settings, &mut outputs)
                        .build(),
                    setting.password.clone(),
                    setting.address.clone(),
                )?;
                tasks.add_task(task_loop!(source));
            }
            SourceType::Bresser6in1(setting) => {
                let mut source = Bresser6in1Source::new(
                    base_builder
                        .name(source.name.clone())
                        .interval(Duration::from_secs(setting.poll_interval))
                        .add_processor(&source.name, settings, &mut outputs)
                        .build(),
                );
                tasks.add_task(task_loop!(source));
            }
        }
    }

    if !tasks.has_tasks() {
        debug!(logger, "No sources enabled, using dummy");
        let mut dummy = DummySource::new(
            SourceBaseBuilder::new(influx_client, tasks.cancel_rx(), logger)
                .name("dummy".into())
                .interval(Duration::from_secs(86400))
                .build(),
        );
        tasks.add_task(task_loop!(dummy));
    }

    Ok((tasks.build(), outputs))
}

fn sleep_duration(
    interval: u64,
    oversample_factor: u64,
    now: u64,
) -> (Duration, bool) {
    let main_duration = Duration::from_secs(interval - (now % interval));

    if oversample_factor <= 1 {
        return (main_duration, false);
    }

    let sample_interval = interval / oversample_factor;
    let next_duration =
        Duration::from_secs(sample_interval - (now % sample_interval));

    (next_duration, next_duration != main_duration)
}

pub async fn sleep_aligned(
    interval: Duration,
    oversample_factor: u64,
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
    let (sleep_time, oversample) =
        sleep_duration(interval_s, oversample_factor, now.as_secs());
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
            return Ok(TaskState::Running(TaskTiming::new(now, oversample)));
        }
        else => {
            return Err("sleep_aligned returned".into());
        }
    }
}

#[test]
fn test_sleep_duration() {
    assert_eq!(
        (Duration::from_secs(57), false),
        sleep_duration(300, 1, 1621753443)
    );
    assert_eq!(
        (Duration::from_secs(57), false),
        sleep_duration(300, 0, 1621753443)
    );
    assert_eq!(
        (Duration::from_secs(30), false),
        sleep_duration(60, 1, 1621754070)
    );
    assert_eq!(
        (Duration::from_secs(10), true),
        sleep_duration(60, 3, 1621754070)
    );
    assert_eq!(
        (Duration::from_secs(19), false),
        sleep_duration(60, 3, 1621754081)
    );
}
