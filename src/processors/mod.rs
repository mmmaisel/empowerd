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
use crate::models::Model;
use crate::settings::{ProcessorType, Settings};
use crate::sinks::ArcSink;
use crate::task_group::{TaskGroup, TaskGroupBuilder, TaskState};
use crate::task_loop;
use slog::{debug, Logger};
use std::collections::BTreeMap;
use tokio::sync::watch;

mod available_power;
mod charging;
mod debug;
mod dummy;

pub use available_power::AvailablePowerProcessor;
pub use charging::ChargingProcessor;
pub use debug::DebugProcessor;
pub use dummy::DummyProcessor;

pub struct ProcessorBase {
    name: String,
    canceled: watch::Receiver<TaskState>,
    logger: Logger,
}

impl ProcessorBase {
    pub fn new(
        name: String,
        canceled: watch::Receiver<TaskState>,
        logger: Logger,
    ) -> Self {
        Self {
            name,
            canceled,
            logger,
        }
    }
}

pub fn processor_tasks(
    logger: Logger,
    settings: &Settings,
    mut inputs: BTreeMap<String, watch::Receiver<Model>>,
    sinks: BTreeMap<String, ArcSink>,
) -> Result<TaskGroup, String> {
    let tasks = TaskGroupBuilder::new("processors".into(), logger.clone());
    let mut outputs = BTreeMap::<String, watch::Sender<Model>>::new();

    for processor in &settings.processors {
        let (tx, rx) = watch::channel(Model::None);
        inputs.insert(processor.name.clone(), rx);
        outputs.insert(processor.name.clone(), tx);
    }

    for processor in &settings.processors {
        match &processor.variant {
            ProcessorType::AvailablePower(setting) => {
                let battery_source = match inputs.get(&setting.battery_input) {
                    Some(x) => x.clone(),
                    None => {
                        return Err(format!(
                            "Missing battery input for Processor {}",
                            &processor.name
                        ))
                    }
                };
                let meter_source = match inputs.get(&setting.meter_input) {
                    Some(x) => x.clone(),
                    None => {
                        return Err(format!(
                            "Missing meter input for Processor {}",
                            &processor.name
                        ))
                    }
                };
                let power_output = match outputs.remove(&processor.name) {
                    Some(x) => x,
                    None => {
                        return Err(format!(
                            "Missing power output for Processor {}",
                            &processor.name
                        ))
                    }
                };
                let mut processor = AvailablePowerProcessor::new(
                    ProcessorBase::new(
                        processor.name.clone(),
                        tasks.cancel_rx(),
                        logger.clone(),
                    ),
                    battery_source,
                    meter_source,
                    power_output,
                );
                tasks.add_task(task_loop!(processor));
            }
            ProcessorType::Debug(setting) => {
                let sink = match sinks.get(&setting.output) {
                    Some(x) => x.to_owned(),
                    None => {
                        return Err(format!(
                            "Missing sink for Processor {}",
                            &processor.name
                        ))
                    }
                };
                let source = match inputs.get(&setting.input) {
                    Some(x) => x.clone(),
                    None => {
                        return Err(format!(
                            "Missing source for Processor {}",
                            &processor.name
                        ))
                    }
                };
                if let ArcSink::Debug(sink) = sink {
                    let mut processor = DebugProcessor::new(
                        ProcessorBase::new(
                            processor.name.clone(),
                            tasks.cancel_rx(),
                            logger.clone(),
                        ),
                        source,
                        sink,
                    );
                    tasks.add_task(task_loop!(processor));
                } else {
                    return Err(
                        "Unsupported sink type for DebugProcessor".into()
                    );
                }
            }
            ProcessorType::Charging(setting) => {
                let power_source = match inputs.get(&setting.power_input) {
                    Some(x) => x.clone(),
                    None => {
                        return Err(format!(
                            "Missing power input for Processor {}",
                            &processor.name
                        ))
                    }
                };
                let power_sink: watch::Sender<Model> =
                    match outputs.remove(&processor.name) {
                        Some(x) => x,
                        None => {
                            return Err(format!(
                                "Missing power output for Processor {}",
                                &processor.name
                            ))
                        }
                    };
                let wallbox_source = match inputs.get(&setting.wallbox_input) {
                    Some(x) => x.clone(),
                    None => {
                        return Err(format!(
                            "Missing wallbox source for Processor {}",
                            &processor.name
                        ))
                    }
                };
                let wallbox_sink = match sinks.get(&setting.wallbox_output) {
                    Some(x) => x.to_owned(),
                    None => {
                        return Err(format!(
                            "Missing sink 'wallbox_output' for Processor {}",
                            &processor.name
                        ))
                    }
                };
                if let ArcSink::KeContact(wallbox_sink) = wallbox_sink {
                    let mut processor = ChargingProcessor::new(
                        ProcessorBase::new(
                            processor.name.clone(),
                            tasks.cancel_rx(),
                            logger.clone(),
                        ),
                        power_source,
                        wallbox_source,
                        power_sink,
                        wallbox_sink,
                        setting.tau,
                    );
                    tasks.add_task(task_loop!(processor));
                } else {
                    return Err(
                        "Unsupported sink type for ChargingProcessor".into()
                    );
                }
            }
        }
    }

    if !tasks.has_tasks() {
        debug!(logger, "No processors enabled, using dummy");
        let mut dummy = DummyProcessor::new(ProcessorBase::new(
            "dummy".into(),
            tasks.cancel_rx(),
            logger,
        ));
        tasks.add_task(task_loop!(dummy));
    }

    Ok(tasks.build())
}
