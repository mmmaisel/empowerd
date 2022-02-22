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

mod debug;

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
    sinks: BTreeMap<String, ArcSink>,
) -> Result<(TaskGroup, BTreeMap<String, watch::Sender<Model>>), String> {
    let mut inputs = BTreeMap::<String, watch::Sender<Model>>::new();
    let tasks = TaskGroupBuilder::new("processors".into(), logger.clone());

    for processor in &settings.processors {
        match processor {
            Processor::Debug(settings) => {
                let sink = match sinks.get(&settings.output) {
                    Some(x) => x.to_owned(),
                    None => {
                        return Err(format!(
                            "Missing sink for Processor {}",
                            &settings.name
                        ))
                    }
                };
                if let ArcSink::Debug(sink) = sink {
                    let (output, input) = watch::channel(Model::None);
                    let mut debug = DebugProcessor::new(
                        ProcessorBase::new(
                            settings.name.clone(),
                            tasks.cancel_rx(),
                            logger.clone(),
                        ),
                        input,
                        sink,
                    );
                    inputs.insert(settings.input.clone(), output);
                    tasks.add_task(task_loop!(debug));
                } else {
                    return Err(
                        "Unsupported sink type for DebugProcessor".into()
                    );
                }
            }
        }
    }

    Ok((tasks.build(), inputs))
}
