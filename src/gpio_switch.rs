/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2021 Max Maisel

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
use super::settings::Gpio;
use gpio_cdev::{Chip, Line, LineHandle, LineRequestFlags};

#[derive(Debug)]
struct Channel {
    pub line: Line,
    pub pin: LineHandle,
    pub name: String,
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
    pub fn new(gpios: Vec<Gpio>) -> Result<Self, String> {
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
                });
            })
            .collect::<Result<Vec<Channel>, String>>()?;

        return Ok(Self { channels });
    }

    pub fn set_open(&self, channel: usize, open: bool) -> Result<(), String> {
        match self.channels.get(channel) {
            Some(channel) => {
                match channel.pin.set_value(if open { 0 } else { 1 }) {
                    Ok(_) => return Ok(()),
                    Err(e) => return Err(e.to_string()),
                }
            }
            None => return Err("Switch index not found".into()),
        };
    }

    pub fn get_names(&self) -> Vec<String> {
        return self
            .channels
            .iter()
            .map(|channel| channel.name.clone())
            .collect();
    }

    pub fn get_name(&self, channel: usize) -> Result<String, String> {
        match self.channels.get(channel) {
            Some(channel) => return Ok(channel.name.clone()),
            None => return Err("Switch index not found".into()),
        };
    }

    pub fn get_open(&self) -> Result<Vec<bool>, String> {
        return self
            .channels
            .iter()
            .map(|channel| {
                return match channel.pin.get_value() {
                    Ok(x) => Ok(x == 0),
                    Err(e) => Err(e.to_string()),
                };
            })
            .collect();
    }
}
