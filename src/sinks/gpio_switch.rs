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
use crate::settings::Icon;
use gpio_cdev::{Chip, Line, LineHandle, LineRequestFlags};

#[derive(Debug)]
pub struct Channel {
    pub line: Line,
    pub pin: LineHandle,
    pub name: String,
    pub icon: Icon,
}

pub struct GpioCreateInfo {
    pub name: String,
    pub icon: Icon,
    pub dev: String,
    pub num: u32,
}

#[derive(Debug)]
pub struct GpioSwitch {
    channels: Vec<Channel>,
}

impl Drop for GpioSwitch {
    fn drop(&mut self) {
        let lines: Vec<Line> =
            self.channels.drain(..).map(|c| c.line).collect();
        for line in lines {
            if let Err(e) = line.request(LineRequestFlags::INPUT, 0, "") {
                panic!("Failed to uninitialize pin: {}", e)
            }
        }
    }
}

impl GpioSwitch {
    pub fn new(gpios: Vec<GpioCreateInfo>) -> Result<Self, String> {
        let channels = gpios
            .into_iter()
            .map(|gpio| {
                let mut chip = Chip::new(&gpio.dev).map_err(|e| {
                    format!("Could not open {}: {}", &gpio.dev, e)
                })?;
                let line = chip.get_line(gpio.num).map_err(|e| {
                    format!("Could not open gpio {}: {}", &gpio.num, e)
                })?;
                let pin = line
                    .request(LineRequestFlags::OUTPUT, 1, &gpio.name)
                    .map_err(|e| {
                        format!(
                            "Could not get handle for gpio {}: {}",
                            &gpio.num, e
                        )
                    })?;
                return Ok(Channel {
                    pin,
                    line,
                    name: gpio.name,
                    icon: gpio.icon,
                });
            })
            .collect::<Result<Vec<Channel>, String>>()?;

        return Ok(Self { channels });
    }

    pub fn set_open(&self, id: usize, open: bool) -> Result<(), String> {
        let channel = self.get_channel(id)?;
        return self.set_open_raw(channel, open);
    }

    pub fn set_open_raw(
        &self,
        channel: &Channel,
        open: bool,
    ) -> Result<(), String> {
        if let Err(e) = channel.pin.set_value(if open { 0 } else { 1 }) {
            return Err(e.to_string());
        }

        Ok(())
    }

    pub fn get_channel(&self, id: usize) -> Result<&Channel, String> {
        self.channels
            .get(id)
            .ok_or(format!("Channel index '{}' not found", id))
    }

    pub fn get_ids(&self) -> Vec<usize> {
        return self
            .channels
            .iter()
            .enumerate()
            .map(|(idx, _channel)| idx)
            .collect();
    }

    pub fn get_id_by_name(&self, name: &str) -> Result<usize, String> {
        self.channels
            .iter()
            .position(|x| x.name == name)
            .ok_or(format!("Switch with name '{}' does not exist.", name))
    }

    pub fn get_name(&self, id: usize) -> Result<String, String> {
        let channel = self.get_channel(id)?;
        Ok(channel.name.clone())
    }

    pub fn get_icon(&self, id: usize) -> Result<String, String> {
        let channel = self.get_channel(id)?;
        Ok(channel.icon.to_string())
    }

    pub fn get_open(&self, id: usize) -> Result<bool, String> {
        let channel = self.get_channel(id)?;
        match channel.pin.get_value() {
            Ok(x) => Ok(x == 0),
            Err(e) => Err(e.to_string()),
        }
    }
}
