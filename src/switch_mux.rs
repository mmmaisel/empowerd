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
use crate::{settings::Icon, sinks::GpioSwitch};
use async_trait::async_trait;
use std::{collections::BTreeMap, fmt::Debug, path::PathBuf, sync::Arc};
use tokio::sync::watch;

#[async_trait]
pub trait SwitchGroup: Debug + Send + Sync {
    async fn read_val(&self, idx: usize) -> Result<bool, String>;
    async fn write_val(&self, idx: usize, val: bool) -> Result<(), String>;
}

#[derive(Debug)]
pub struct SwitchArgs {
    pub num: usize,
    pub name: String,
    pub icon: Icon,
    pub proc: Option<watch::Sender<bool>>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SwitchType {
    Gpio { dev: PathBuf },
}

#[derive(Debug)]
struct Channel {
    pub name: String,
    pub icon: Icon,
    pub idx: usize,
    pub proc: Option<watch::Sender<bool>>,
    pub switch: Arc<dyn SwitchGroup>,
}

#[derive(Debug)]
pub struct SwitchMux {
    channels: Vec<Channel>,
}

impl SwitchMux {
    pub fn new(
        config: BTreeMap<SwitchType, Vec<SwitchArgs>>,
    ) -> Result<Self, String> {
        // TODO: keep order from cfg file

        let mut channels = Vec::new();
        for (typ, args) in config {
            match typ {
                SwitchType::Gpio { dev } => {
                    let switch = Arc::new(GpioSwitch::new(
                        &dev,
                        &args.iter().map(|x| x.num as u32).collect::<Vec<_>>(),
                    )?);
                    for arg in args {
                        channels.push(Channel {
                            name: arg.name,
                            icon: arg.icon,
                            idx: arg.num,
                            proc: arg.proc,
                            switch: switch.clone(),
                        });
                    }
                }
            }
        }

        Ok(Self { channels })
    }

    fn get_channel(&self, id: usize) -> Result<&Channel, String> {
        self.channels
            .get(id)
            .ok_or(format!("Channel {id} not found"))
    }

    pub fn len(&self) -> usize {
        self.channels.len()
    }

    pub fn ids(&self) -> Vec<usize> {
        return self
            .channels
            .iter()
            .enumerate()
            .map(|(idx, _channel)| idx)
            .collect();
    }

    pub fn id_by_name(&self, name: &str) -> Result<usize, String> {
        self.channels
            .iter()
            .position(|x| x.name == name)
            .ok_or(format!("Switch with name '{}' does not exist.", name))
    }

    pub fn name(&self, id: usize) -> Result<String, String> {
        let channel = self.get_channel(id)?;
        Ok(channel.name.clone())
    }

    pub fn icon(&self, id: usize) -> Result<String, String> {
        let channel = self.get_channel(id)?;
        Ok(channel.icon.to_string())
    }

    pub async fn read_val(&self, id: usize) -> Result<bool, String> {
        let channel = self.get_channel(id)?;
        channel
            .switch
            .read_val(channel.idx)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn write_val(&self, id: usize, val: bool) -> Result<(), String> {
        let channel = self.get_channel(id)?;
        if let Some(proc) = &channel.proc {
            proc.send_replace(val);
        } else {
            channel.switch.write_val(channel.idx, val).await?;
        }

        Ok(())
    }

    pub async fn write_val_raw(
        &self,
        id: usize,
        val: bool,
    ) -> Result<(), String> {
        let channel = self.get_channel(id)?;
        channel.switch.write_val(channel.idx, val).await
    }
}
