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
use getopts::Options;
use serde::Deserialize;
use std::env;

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Database {
    pub url: String,
    pub name: String,
    pub user: String,
    pub password: String,
}

impl Default for Database {
    fn default() -> Self {
        Self {
            url: "127.0.0.1:8086".into(),
            name: "empowerd".into(),
            user: "empowerd".into(),
            password: "password".into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct GraphQL {
    pub listen_address: String,
    pub session_timeout: u64,
    pub username: String,
    pub hashed_password: String,
}

impl Default for GraphQL {
    fn default() -> Self {
        Self {
            listen_address: "0.0.0.0:3001".into(),
            session_timeout: 300,
            username: "user".into(),
            hashed_password: "!".into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct SunnyBoyStorage {
    pub name: String,
    pub address: String,
    pub poll_interval: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SunnyIsland {
    pub name: String,
    pub address: String,
    pub poll_interval: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SunspecSolar {
    pub name: String,
    pub address: String,
    pub modbus_id: Option<u8>,
    pub poll_interval: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DachsMsrS {
    pub name: String,
    pub address: String,
    pub password: String,
    pub poll_interval: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SmlMeter {
    pub name: String,
    pub device: String,
    pub baud: u32,
    pub poll_interval: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SunnyBoySpeedwire {
    pub name: String,
    pub address: String,
    pub password: String,
    pub poll_interval: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Bresser6in1 {
    pub name: String,
    pub poll_interval: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Source {
    SunnyBoyStorage(SunnyBoyStorage),
    SunnyIsland(SunnyIsland),
    SunspecSolar(SunspecSolar),
    DachsMsrS(DachsMsrS),
    SmlMeter(SmlMeter),
    SunnyBoySpeedwire(SunnyBoySpeedwire),
    Bresser6in1(Bresser6in1),
}

#[derive(Clone, Debug, Deserialize)]
pub struct Gpio {
    pub name: String,
    pub dev: String,
    #[serde(rename = "pin_num")]
    pub num: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Sink {
    Gpio(Gpio),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub daemonize: bool,
    pub pid_file: String,
    pub wrk_dir: String,
    pub logfile: String,
    pub log_level: u8,
    pub database: Database,
    pub graphql: GraphQL,

    #[serde(rename = "source")]
    pub sources: Vec<Source>,
    #[serde(rename = "sink")]
    pub sinks: Vec<Sink>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            daemonize: false,
            pid_file: "/run/empowerd/pid".into(),
            wrk_dir: "/".into(),
            logfile: "/var/log/empowerd.log".into(),
            log_level: 0,
            database: Database::default(),
            graphql: GraphQL::default(),
            sources: Vec::new(),
            sinks: Vec::new(),
        }
    }
}

impl Settings {
    pub fn load_from_file(filename: &str) -> Result<Settings, String> {
        let toml = std::fs::read_to_string(filename).map_err(|e| {
            format!("Could not read config file '{}': {}", &filename, e)
        })?;
        return toml::from_str(&toml)
            .map_err(|e| format!("Could not parse config: {}", e));
    }

    pub fn load() -> Result<Settings, String> {
        let mut options = Options::new();
        options.optopt("c", "", "config filename", "NAME").optflag(
            "d",
            "",
            "daemonize",
        );

        let matches = options.parse(env::args()).map_err(|e| e.to_string())?;

        let cfg_path = match matches.opt_str("c") {
            Some(x) => x,
            None => "/etc/empowerd/empowerd.conf".into(),
        };

        let mut settings = Settings::load_from_file(&cfg_path)?;

        if matches.opt_present("d") {
            settings.daemonize = true;
        }

        return Ok(settings);
    }
}
