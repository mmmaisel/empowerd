use std::convert::TryInto;
use std::thread;
use std::time;
use sysfs_gpio::{Direction, Pin};

struct Channel {
    pub pin: Pin,
    pub name: String,
}

pub struct WaterSwitch {
    channels: Vec<Channel>,
}

impl Drop for WaterSwitch {
    fn drop(&mut self) {
        for channel in &self.channels {
            if let Err(e) = channel.pin.set_direction(Direction::In) {
                panic!("Failed to uninitialize pin: {}", e)
            }
            if let Err(e) = channel.pin.unexport() {
                panic!("Failed to uninitialize pin: {}", e)
            }
        }
    }
}

impl WaterSwitch {
    pub fn new(
        pin_nums: Vec<i64>,
        pin_names: Vec<String>,
    ) -> Result<WaterSwitch, String> {
        let channels = pin_nums
            .into_iter()
            .zip(pin_names.into_iter())
            .map(|(pin_num, pin_name)| {
                let pin_num = TryInto::<u64>::try_into(pin_num)
                    .map_err(|e| e.to_string())?;
                let pin = Pin::new(pin_num);
                pin.export().map_err(|e| e.to_string())?;
                thread::sleep(time::Duration::from_millis(100));
                pin.set_direction(Direction::Out)
                    .map_err(|e| e.to_string())?;
                pin.set_value(1).map_err(|e| e.to_string())?;
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
