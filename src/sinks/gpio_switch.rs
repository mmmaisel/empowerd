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
use gpiocdev::{line::Value, request::Config, Request};
use tokio::sync::watch;

#[derive(Debug)]
pub struct Channel {
    pub offset: u32,
    pub pin: Request,
    pub name: String,
    pub icon: Icon,
    pub processor: Option<watch::Sender<bool>>,
}

pub struct GpioCreateInfo {
    pub name: String,
    pub icon: Icon,
    pub dev: String,
    pub num: u32,
    pub processor: Option<watch::Sender<bool>>,
}

#[derive(Debug)]
pub struct GpioSwitch {
    channels: Vec<Channel>,
}

impl Drop for GpioSwitch {
    fn drop(&mut self) {
        let pins: Vec<Request> =
            self.channels.drain(..).map(|c| c.pin).collect();
        for pin in pins {
            let mut config = pin.config();
            if let Err(e) = pin.reconfigure(config.as_input()) {
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
                let mut config = Config::default();
                config
                    .with_line(gpio.num)
                    .as_active_low()
                    .as_output(Value::Inactive);
                let pin = Request::from_config(config)
                    .on_chip(&gpio.dev)
                    .with_consumer(&gpio.name)
                    .request()
                    .map_err(|e| {
                        format!(
                            "Could not open {}:{}: {}",
                            &gpio.dev, &gpio.num, e
                        )
                    })?;
                return Ok(Channel {
                    offset: gpio.num,
                    pin,
                    name: gpio.name,
                    icon: gpio.icon,
                    processor: gpio.processor,
                });
            })
            .collect::<Result<Vec<Channel>, String>>()?;

        return Ok(Self { channels });
    }

    pub fn set_open(&self, id: usize, open: bool) -> Result<(), String> {
        let channel = self.get_channel(id)?;
        if let Some(processor) = &channel.processor {
            processor.send_replace(open);
        } else {
            return self.set_open_raw(channel, open);
        }

        Ok(())
    }

    pub fn set_open_raw(
        &self,
        channel: &Channel,
        open: bool,
    ) -> Result<(), String> {
        if let Err(e) = channel.pin.set_value(channel.offset, open.into()) {
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
        match channel.pin.value(channel.offset) {
            Ok(x) => Ok(x.into()),
            Err(e) => Err(e.to_string()),
        }
    }
}
