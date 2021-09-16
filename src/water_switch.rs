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
use gpio_cdev::{Chip, LineHandle, LineRequestFlags};

#[derive(Debug)]
struct Channel {
    pub pin: LineHandle,
    pub name: String,
}

#[derive(Debug)]
pub struct WaterSwitch {
    channels: Vec<Channel>,
}

impl WaterSwitch {
    pub fn new(
        gpiodev: &str,
        pin_nums: Vec<u32>,
        pin_names: Vec<String>,
    ) -> Result<WaterSwitch, String> {
        let mut chip = Chip::new(gpiodev)
            .map_err(|e| format!("Could not open {}: {}", gpiodev, e))?;
        let channels = pin_nums
            .into_iter()
            .zip(pin_names.into_iter())
            .map(|(pin_num, pin_name)| {
                let line = chip.get_line(pin_num).map_err(|e| {
                    format!("Could not open pin {}: {}", pin_num, e)
                })?;
                let pin = line
                    .request(LineRequestFlags::OUTPUT, 1, &pin_name)
                    .map_err(|e| {
                        format!(
                            "Could not get handle for pin {}: {}",
                            pin_num, e
                        )
                    })?;
                return Ok(Channel {
                    pin: pin,
                    name: pin_name,
                });
            })
            .collect::<Result<Vec<Channel>, String>>()?;

        return Ok(WaterSwitch { channels: channels });
    }

    pub fn set_open(&self, channel: usize, open: bool) -> Result<(), String> {
        match self.channels.get(channel) {
            Some(channel) => {
                match channel.pin.set_value(if open { 0 } else { 1 }) {
                    Ok(_) => return Ok(()),
                    Err(e) => return Err(e.to_string()),
                }
            }
            None => return Err("Valve index not found".into()),
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
            None => return Err("Valve index not found".into()),
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
