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
use getopts::Options;
use serde::Deserialize;
use std::collections::BTreeSet;
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
pub struct DebugSource {
    pub poll_interval: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SunnyBoyStorage {
    pub address: String,
    pub poll_interval: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SunnyIsland {
    pub address: String,
    pub poll_interval: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SunspecSolar {
    pub address: String,
    pub modbus_id: Option<u8>,
    pub poll_interval: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DachsMsrS {
    pub address: String,
    pub password: String,
    pub poll_interval: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct KeContact {
    pub address: String,
    pub poll_interval: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SmlMeter {
    pub device: String,
    pub baud: u32,
    pub poll_interval: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SunnyBoySpeedwire {
    pub address: String,
    pub password: String,
    pub poll_interval: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Bresser6in1 {
    pub poll_interval: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum SourceType {
    Debug(DebugSource),
    SunnyBoyStorage(SunnyBoyStorage),
    SunnyIsland(SunnyIsland),
    SunspecSolar(SunspecSolar),
    DachsMsrS(DachsMsrS),
    KeContact(KeContact),
    SmlMeter(SmlMeter),
    SunnyBoySpeedwire(SunnyBoySpeedwire),
    Bresser6in1(Bresser6in1),
}

#[derive(Clone, Debug, Deserialize)]
pub struct Source {
    pub name: String,
    #[serde(flatten)]
    pub variant: SourceType,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DebugProcessor {
    pub input: String,
    pub output: String,
}

impl DebugProcessor {
    fn has_source(&self, source: &str) -> bool {
        self.input == source
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct AvailablePowerProcessor {
    pub battery_input: String,
    pub meter_input: String,
}

impl AvailablePowerProcessor {
    fn has_source(&self, source: &str) -> bool {
        self.meter_input == source || self.battery_input == source
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChargingProcessor {
    pub power_input: String,
    pub wallbox_input: String,
    pub wallbox_output: String,
    pub tau: f64,
}

impl ChargingProcessor {
    fn has_source(&self, source: &str) -> bool {
        self.power_input == source || self.wallbox_input == source
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ProcessorType {
    Debug(DebugProcessor),
    AvailablePower(AvailablePowerProcessor),
    Charging(ChargingProcessor),
}

#[derive(Clone, Debug, Deserialize)]
pub struct Processor {
    pub name: String,
    #[serde(flatten)]
    pub variant: ProcessorType,
}

#[derive(Clone, Debug, Deserialize)]
pub enum Icon {
    Valve,
}

impl std::fmt::Display for Icon {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let name = match self {
            Icon::Valve => "Valve",
        };
        write!(f, "{}", name)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Gpio {
    pub icon: Icon,
    pub dev: String,
    #[serde(rename = "pin_num")]
    pub num: u32,
    #[serde(default = "Gpio::max_on_time")]
    pub on_time: u64,
}

impl Gpio {
    pub fn max_on_time() -> u64 {
        u64::MAX
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct KeContactSink {
    pub address: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum SinkType {
    Debug,
    Gpio(Gpio),
    KeContact(KeContactSink),
}

#[derive(Clone, Debug, Deserialize)]
pub struct Sink {
    pub name: String,
    #[serde(flatten)]
    pub variant: SinkType,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub daemonize: bool,
    pub pid_file: String,
    pub wrk_dir: String,
    pub logfile: String,
    pub log_level: sloggers::types::Severity,
    pub database: Database,
    pub graphql: GraphQL,

    #[serde(rename = "source")]
    pub sources: Vec<Source>,
    #[serde(rename = "processor")]
    pub processors: Vec<Processor>,
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
            log_level: sloggers::types::Severity::Info,
            database: Database::default(),
            graphql: GraphQL::default(),
            sources: Vec::new(),
            processors: Vec::new(),
            sinks: Vec::new(),
        }
    }
}

impl Settings {
    pub fn has_processor(&self, source: &str) -> bool {
        self.processors.iter().any({
            |x| match &x.variant {
                ProcessorType::Debug(x) => x.has_source(source),
                ProcessorType::AvailablePower(x) => x.has_source(source),
                ProcessorType::Charging(x) => x.has_source(source),
            }
        })
    }

    pub fn load_from_file(filename: &str) -> Result<Settings, String> {
        let toml = std::fs::read_to_string(filename).map_err(|e| {
            format!("Could not read config file '{}': {}", &filename, e)
        })?;
        return toml::from_str(&toml)
            .map_err(|e| format!("Could not parse config: {}", e));
    }

    fn validate_name<'a>(
        names: &mut BTreeSet<&'a str>,
        name: &'a str,
    ) -> Result<(), String> {
        if names.contains(&name) {
            return Err(format!("Duplicate name in config file: '{}'", &name));
        } else if name.starts_with("_") {
            return Err(format!("Names must not start with '_': '{}'", &name));
        }
        names.insert(name);
        Ok(())
    }

    fn validate(&self) -> Result<(), String> {
        let mut names = BTreeSet::new();

        for source in &self.sources {
            Self::validate_name(&mut names, &source.name)?;
        }

        for processor in &self.processors {
            Self::validate_name(&mut names, &processor.name)?;
        }

        for sink in &self.sinks {
            Self::validate_name(&mut names, &sink.name)?;
        }

        Ok(())
    }

    pub fn load() -> Result<Settings, String> {
        let mut options = Options::new();
        options.optopt("c", "", "config filename", "NAME").optflag(
            "d",
            "",
            "nodaemonize",
        );

        let matches = options.parse(env::args()).map_err(|e| e.to_string())?;

        let cfg_path = match matches.opt_str("c") {
            Some(x) => x,
            None => "/etc/empowerd/empowerd.conf".into(),
        };

        let mut settings = Settings::load_from_file(&cfg_path)?;

        if matches.opt_present("d") {
            settings.daemonize = false;
        }

        settings.validate()?;
        return Ok(settings);
    }
}
