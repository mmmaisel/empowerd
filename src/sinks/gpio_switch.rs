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
use crate::SwitchGroup;
use gpiocdev::{line::Value, request::Config, Request};
use std::path::Path;

#[derive(Debug)]
pub struct GpioSwitch {
    pins: Request,
}

impl Drop for GpioSwitch {
    fn drop(&mut self) {
        let mut config = self.pins.config();
        if let Err(e) = self.pins.reconfigure(config.as_input()) {
            panic!("Failed to uninitialize pins: {e}")
        }
    }
}

impl GpioSwitch {
    pub fn new(dev: &Path, gpios: &[u32]) -> Result<Self, String> {
        let mut config = Config::default();
        config
            .with_lines(gpios)
            .as_active_low()
            .as_output(Value::Inactive);
        let pins = Request::from_config(config)
            .on_chip(dev)
            .with_consumer("empowerd")
            .request()
            .map_err(|e| format!("Could not open {}: {e}", dev.display()))?;

        Ok(Self { pins })
    }
}

impl SwitchGroup for GpioSwitch {
    fn read_val(&self, idx: usize) -> Result<bool, String> {
        self.pins
            .value(idx as u32)
            .map(|x| x.into())
            .map_err(|e| e.to_string())
    }

    fn write_val(&self, idx: usize, val: bool) -> Result<(), String> {
        self.pins
            .set_value(idx as u32, val.into())
            .map_err(|e| e.to_string())
    }
}
