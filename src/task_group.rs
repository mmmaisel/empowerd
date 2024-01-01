/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2023 Max Maisel

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
use crate::Error;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use slog::{debug, error, info, Logger};
use tokio::sync::watch;
use tokio::task::JoinHandle;

#[derive(Debug, Default)]
pub struct TaskTiming {
    pub now: u64,
    pub oversample: bool,
}

impl TaskTiming {
    pub fn new(now: u64, oversample: bool) -> Self {
        Self { now, oversample }
    }
}

pub type TaskResult = Result<(), Error>;

#[derive(Debug)]
pub enum TaskState {
    Running(TaskTiming),
    Canceled,
}

macro_rules! task_loop {
    ($source:expr) => {
        tokio::task::spawn(async move {
            loop {
                match $source.run().await {
                    Ok(()) => (),
                    Err(crate::Error::Temporary(e)) => {
                        slog::error!($source.logger(), "{}", e)
                    }
                    Err(e) => return e,
                }
            }
        })
    };
}

pub(crate) use task_loop;

pub struct TaskGroupBuilder {
    name: String,
    logger: Logger,
    tasks: FuturesUnordered<JoinHandle<Error>>,
    cancel_tx: watch::Sender<TaskState>,
    cancel_rx: watch::Receiver<TaskState>,
}

impl TaskGroupBuilder {
    pub fn new(name: String, logger: Logger) -> Self {
        let (cancel_tx, cancel_rx) =
            watch::channel(TaskState::Running(TaskTiming::default()));
        Self {
            name,
            logger,
            tasks: FuturesUnordered::new(),
            cancel_tx,
            cancel_rx,
        }
    }

    pub fn add_task(&self, task: JoinHandle<Error>) {
        self.tasks.push(task);
    }

    pub fn cancel_rx(&self) -> watch::Receiver<TaskState> {
        self.cancel_rx.clone()
    }

    pub fn has_tasks(&self) -> bool {
        !self.tasks.is_empty()
    }

    pub fn build(self) -> TaskGroup {
        TaskGroup::new(self.name, self.logger, self.tasks, self.cancel_tx)
    }
}

pub struct TaskGroup {
    name: String,
    logger: Logger,
    tasks: FuturesUnordered<JoinHandle<Error>>,
    cancel: watch::Sender<TaskState>,
}

impl TaskGroup {
    pub fn new(
        name: String,
        logger: Logger,
        tasks: FuturesUnordered<JoinHandle<Error>>,
        cancel: watch::Sender<TaskState>,
    ) -> Self {
        Self {
            name,
            logger,
            tasks,
            cancel,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn run(&mut self) -> Result<(), ()> {
        while let Some(join_result) = self.tasks.next().await {
            let result = match join_result {
                Ok(x) => x,
                Err(e) => {
                    error!(self.logger, "Join task failed: {}", e);
                    return Err(());
                }
            };

            if let Error::Canceled(name) = result {
                debug!(self.logger, "Task '{}' was canceled", name);
            } else {
                error!(self.logger, "Task failed: {:?}", result);
                return Err(());
            }
        }

        Ok(())
    }

    pub fn cancel(&mut self) -> Result<(), String> {
        if self.cancel.is_closed() {
            return Ok(());
        }

        match self.cancel.send(TaskState::Canceled) {
            Ok(_) => {
                info!(self.logger, "Task group '{}' canceled", self.name);
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        }
    }
}
