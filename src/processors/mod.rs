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
use crate::settings::{Processor, Settings};
use crate::sinks::ArcSink;
use crate::task_group::{TaskGroup, TaskGroupBuilder, TaskState};
use crate::task_loop;
use slog::Logger;
use std::collections::BTreeMap;
use tokio::sync::watch;

mod charging;
mod debug;

pub use charging::ChargingProcessor;
pub use debug::DebugProcessor;

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
    inputs: BTreeMap<String, watch::Receiver<Model>>,
    sinks: BTreeMap<String, ArcSink>,
) -> Result<TaskGroup, String> {
    let tasks = TaskGroupBuilder::new("processors".into(), logger.clone());

    for processor in &settings.processors {
        match processor {
            Processor::Debug(setting) => {
                let sink = match sinks.get(&setting.output) {
                    Some(x) => x.to_owned(),
                    None => {
                        return Err(format!(
                            "Missing sink for Processor {}",
                            &setting.name
                        ))
                    }
                };
                let source = match inputs.get(&setting.input) {
                    Some(x) => x.clone(),
                    None => {
                        return Err(format!(
                            "Missing source for Processor {}",
                            &setting.name
                        ))
                    }
                };
                if let ArcSink::Debug(sink) = sink {
                    let mut processor = DebugProcessor::new(
                        ProcessorBase::new(
                            setting.name.clone(),
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
            Processor::Charging(setting) => {
                let sink = match sinks.get(&setting.wallbox_output) {
                    Some(x) => x.to_owned(),
                    None => {
                        return Err(format!(
                            "Missing sink 'wallbox_output' for Processor {}",
                            &setting.name
                        ))
                    }
                };
                let meter_source = match inputs.get(&setting.meter_input) {
                    Some(x) => x.clone(),
                    None => {
                        return Err(format!(
                            "Missing meter for Processor {}",
                            &setting.name
                        ))
                    }
                };
                let battery_source = match inputs.get(&setting.battery_input) {
                    Some(x) => x.clone(),
                    None => {
                        return Err(format!(
                            "Missing battery source for Processor {}",
                            &setting.name
                        ))
                    }
                };
                let wallbox_source = match inputs.get(&setting.wallbox_input) {
                    Some(x) => x.clone(),
                    None => {
                        return Err(format!(
                            "Missing wallbox source for Processor {}",
                            &setting.name
                        ))
                    }
                };
                if let ArcSink::KeContact(sink) = sink {
                    let mut processor = ChargingProcessor::new(
                        ProcessorBase::new(
                            setting.name.clone(),
                            tasks.cancel_rx(),
                            logger.clone(),
                        ),
                        meter_source,
                        battery_source,
                        wallbox_source,
                        sink,
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

    Ok(tasks.build())
}
