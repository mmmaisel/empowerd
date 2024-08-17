/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2024 Max Maisel

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
#![forbid(unsafe_code)]
#![allow(clippy::needless_return)]
#![allow(clippy::redundant_field_names)]

pub mod error;
pub mod graphql;
pub mod misc;
pub mod models;
pub mod multi_setpoint_hysteresis;
pub mod processors;
pub mod pt1;
pub mod seasonal;
pub mod session_manager;
pub mod settings;
pub mod sinks;
pub mod sources;
pub mod task_group;
pub mod tri_state;

use error::Error;
use processors::ProcessorCommands;
use session_manager::SessionManager;
use sinks::GpioSwitch;
use slog::Logger;
use std::sync::Arc;

#[derive(Debug)]
pub struct Globals {
    pub logger: Logger,
    pub username: String,
    pub hashed_pw: String,
    pub session_manager: SessionManager,
    pub gpio_switch: Arc<GpioSwitch>,
    pub processor_cmds: ProcessorCommands,
}

#[derive(Debug)]
pub struct Context {
    pub globals: Arc<Globals>,
    pub token: String,
}

impl juniper::Context for Context {}
