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
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use slog::{debug, error, info, Logger};
use tokio::sync::watch;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub enum TaskResult {
    Running,
    Canceled(String),
    Err(String),
}

#[derive(Debug)]
pub enum TaskState {
    Running(u64),
    Canceled,
}

#[macro_export]
macro_rules! task_loop {
    ($source:expr) => {
        tokio::task::spawn(async move {
            loop {
                let result = $source.run().await;
                if let crate::task_group::TaskResult::Running = result {
                    continue;
                }
                return result;
            }
        })
    };
}

pub struct TaskGroup {
    name: String,
    logger: Logger,
    tasks: FuturesUnordered<JoinHandle<TaskResult>>,
    cancel: watch::Sender<TaskState>,
}

impl TaskGroup {
    pub fn new(
        name: String,
        tasks: FuturesUnordered<JoinHandle<TaskResult>>,
        logger: Logger,
        cancel: watch::Sender<TaskState>,
    ) -> Self {
        Self {
            name,
            logger,
            tasks,
            cancel,
        }
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

            match result {
                TaskResult::Running => {}
                TaskResult::Canceled(name) => {
                    debug!(self.logger, "Task '{}' was canceled", name);
                }
                TaskResult::Err(e) => {
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
        return match self.cancel.send(TaskState::Canceled) {
            Ok(_) => {
                info!(self.logger, "Task group '{}' canceled", self.name);
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        };
    }
}
