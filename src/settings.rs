/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2023 Max Maisel

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
use std::fmt::{self, Debug};

/// Defines the database location and credentials.
#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct Database {
    /// Database address and port
    pub url: String,
    /// Database name
    pub name: String,
    /// Login username
    pub user: String,
    pub password: String,
}

impl Debug for Database {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Database")
            .field("url", &self.url)
            .field("name", &self.name)
            .field("user", &self.user)
            .field("password", &"**SECRET**")
            .finish()
    }
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

/// Defines GraphQL API location and credentials.
#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct GraphQL {
    /// API server listen address and port
    pub listen_address: String,
    /// Session inactivity timeout in seconds
    pub session_timeout: u64,
    /// API username
    pub username: String,
    /// Argon2 hashed password of the API user.
    pub hashed_password: String,
}

impl Debug for GraphQL {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Database")
            .field("listen_address", &self.listen_address)
            .field("session_timeout", &self.session_timeout)
            .field("username", &self.username)
            .field("hashed_password", &"**SECRET**")
            .finish()
    }
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

/// Defines the geographical location of the system.
/// This is required if seasonal corrections are used to calculate
/// current day length.
#[derive(Clone, Debug, Deserialize)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}

/// Dummy data source for debugging.
#[derive(Clone, Debug, Deserialize)]
pub struct DebugSource {
    pub poll_interval: u64,
}

/// SMA SunnyBoyStorage inverter Modbus data source parameters.
#[derive(Clone, Debug, Deserialize)]
pub struct SunnyBoyStorage {
    /// Device IP address and port
    pub address: String,
    /// Data acquisition poll interval
    pub poll_interval: u64,
}

/// SMA SunnyIsland inverter Modbus data source parameters.
#[derive(Clone, Debug, Deserialize)]
pub struct SunnyIsland {
    /// Device IP address and port
    pub address: String,
    /// Data acquisition poll interval
    pub poll_interval: u64,
}

/// Generic Sunspec Modbus compatible inverter data source parameters.
#[derive(Clone, Debug, Deserialize)]
pub struct SunspecSolar {
    /// Device IP address and port
    pub address: String,
    /// Optional modbus device ID. This is only required for some devices.
    pub modbus_id: Option<u8>,
    /// Data acquisition poll interval
    pub poll_interval: u64,
}

/// Senertec Dachs MSR-S generator REST-API data source parameters.
#[derive(Clone, Deserialize)]
pub struct DachsMsrS {
    /// Device IP address and port
    pub address: String,
    /// API password
    pub password: String,
    /// Data acquisition poll interval
    pub poll_interval: u64,
}

impl Debug for DachsMsrS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DachsMsrS")
            .field("address", &self.address)
            .field("password", &"**SECRET**")
            .field("poll_interval", &self.poll_interval)
            .finish()
    }
}

/// Keba KeContact wallbox JSON data source parameters.
#[derive(Clone, Debug, Deserialize)]
pub struct KeContact {
    /// Device IP address and port
    pub address: String,
    /// Data acquisition poll interval
    pub poll_interval: u64,
}

/// Lambda heat pump Modbus data source parameters.
#[derive(Clone, Debug, Deserialize)]
pub struct LambdaHeatPump {
    /// Device IP address and port
    pub address: String,
    /// Data acquisition poll interval
    pub poll_interval: u64,
    /// Data acquisition oversamplinf factor.
    /// Data is sampled every (poll_interval / oversample_factor) seconds,
    /// averaged and processed every poll_interval seconds.
    pub oversample_factor: u64,
}

/// SMA energy meter Speedwire data source parameters.
#[derive(Clone, Debug, Deserialize)]
pub struct SmaMeter {
    /// Device IP address without port
    pub address: String,
    /// Local IP address which will receive broadcast messages.
    pub bind_address: String,
    /// Data acquisition poll interval
    pub poll_interval: u64,
}

/// Generic SML (smart meter language) compatible energy meter data
/// source parameters.
#[derive(Clone, Debug, Deserialize)]
pub struct SmlMeter {
    /// Serial TTY device path
    pub device: String,
    /// Baudrate of the device
    pub baud: u32,
    /// Data acquisition poll interval
    pub poll_interval: u64,
}

/// SMA SonnyBoy inverter Speedwire data source parameters.
#[derive(Clone, Deserialize)]
pub struct SunnyBoySpeedwire {
    /// Device IP address without port
    pub address: String,
    /// Speedwire password
    pub password: String,
    /// Data acquisition poll interval
    pub poll_interval: u64,
}

impl Debug for SunnyBoySpeedwire {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SunnyBoySpeedwire")
            .field("address", &self.address)
            .field("password", &"**SECRET**")
            .field("poll_interval", &self.poll_interval)
            .finish()
    }
}

/// Bresser 6 in 1 USB weather station data source.
#[derive(Clone, Debug, Deserialize)]
pub struct Bresser6in1 {
    /// Data acquisition poll interval
    pub poll_interval: u64,
}

/// Common type for handling different data sources.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum SourceType {
    Debug(DebugSource),
    SunnyBoyStorage(SunnyBoyStorage),
    SunnyIsland(SunnyIsland),
    SunspecSolar(SunspecSolar),
    DachsMsrS(DachsMsrS),
    KeContact(KeContact),
    LambdaHeatPump(LambdaHeatPump),
    SmaMeter(SmaMeter),
    SmlMeter(SmlMeter),
    SunnyBoySpeedwire(SunnyBoySpeedwire),
    Bresser6in1(Bresser6in1),
}

/// Defines a data source node.
#[derive(Clone, Debug, Deserialize)]
pub struct Source {
    /// Name of the data source.
    pub name: String,
    /// The individual data source parameters.
    #[serde(flatten)]
    pub variant: SourceType,
}

/// Applies an offset correction to data based on the current day length.
#[derive(Clone, Debug, Deserialize)]
pub struct Seasonal {
    /// Current day length offset in hours.
    /// This constant is added to the calculated day length before
    /// further processing.
    pub offset: f64,
    /// Day length to target unit conversion factor with the unit X/hour.
    pub gain: f64,
    /// Current date offset in days. This is applied before day length
    /// calculation. Useful to correct usual weather "lag" effects.
    pub phase: i64,
}

/// Dummy processor for debug porposes.
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

/// Calculates the currently available power based on grid exchange power
/// and battery charge.
#[derive(Clone, Debug, Deserialize)]
pub struct AvailablePowerProcessor {
    /// Name of the battery source node to use as input.
    pub battery_input: String,
    /// Name of the energy meter source node to use as input.
    pub meter_input: String,
    /// Battery charge threshold in watt hours for power clearance.
    pub battery_threshold: f64,
    /// Output power lowpass filter time constant.
    pub tau: f64,
}

impl AvailablePowerProcessor {
    fn has_source(&self, source: &str) -> bool {
        self.meter_input == source || self.battery_input == source
    }
}

/// Controls the power consumption of an appliance.
#[derive(Clone, Debug, Deserialize)]
pub struct ApplianceProcessor {
    /// Name of available power input node for the appliance.
    /// Can either be an AvailablePowerProcessor or another ApplianceProcessor.
    pub power_input: String,
    /// Name of the Source node of the appliance.
    pub appliance_input: String,
    /// Name of the Sink node of the appliance.
    pub appliance_output: String,
    /// Retransmit the calculated power every X seconds to the appliance.
    /// This is required for appliances which need periodic retransmits
    /// of the target power in a faster interval than the processing rate.
    #[serde(default = "ApplianceProcessor::default_retransmit_interval")]
    pub retransmit_interval: u64,
    /// Optional seasonal correction for the appliance.
    pub seasonal: Option<Seasonal>,
}

impl ApplianceProcessor {
    fn has_source(&self, source: &str) -> bool {
        self.power_input == source || self.appliance_input == source
    }

    pub fn default_retransmit_interval() -> u64 {
        86400 // one day
    }
}

/// Basic SMA Speedwire energy meter grid exchange load controller.
/// Allows to draw a small constant load from the grid when battery
/// charge depletes.
#[derive(Clone, Debug, Deserialize)]
pub struct LoadControlProcessor {
    /// Address of the source energy meter
    pub meter_addr: String,
    /// Local IP address which will receive broadcast messages.
    pub bind_addr: String,
    /// Serial number of the source energy meter.
    pub meter_serial: u32,
    /// Serial number of the created virtual energy meter.
    pub ctrl_serial: u32,
    /// Name of used the battery source node.
    pub battery_input: String,
    /// Grid power request starts when the battery capacity falls below this
    /// threshold in watt hours.
    pub battery_empty_cap: f64,
    /// Maximum grid power is requested when battery capacity falls below this
    /// threshold in watt hours.
    pub battery_threshold_cap: f64,
    /// Battery capacity hysteresis in watt hours.
    pub hysteresis_cap: f64,
    /// Maximum power in watt that is requested from grid.
    pub basic_load: f64,
    /// Minimum power in watt that is requested from grid.
    pub min_grid_power: f64,
    /// Number of steps between minimum and maximum grid power.
    pub num_points: i32,
    /// Power for grid based battery charging.
    pub charge_power: f64,
    /// Optional seasonal correction.
    pub seasonal: Option<Seasonal>,
}

impl LoadControlProcessor {
    fn has_source(&self, source: &str) -> bool {
        self.battery_input == source
    }
}

/// Common type for handling different data processors.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ProcessorType {
    Debug(DebugProcessor),
    AvailablePower(AvailablePowerProcessor),
    Appliance(ApplianceProcessor),
    LoadControl(LoadControlProcessor),
}

/// Defines a data processor node.
#[derive(Clone, Debug, Deserialize)]
pub struct Processor {
    /// Name of the data processor.
    pub name: String,
    /// The individual data processor parameters.
    #[serde(flatten)]
    pub variant: ProcessorType,
}

/// Available icons for the Web-UI.
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

/// GPIO data sink parameters.
#[derive(Clone, Debug, Deserialize)]
pub struct Gpio {
    /// Icon name for the Web-UI.
    pub icon: Icon,
    /// Name of the controlled device
    pub dev: String,
    /// GPIO pin number
    #[serde(rename = "pin_num")]
    pub num: u32,
    /// Maximum on time of the pin. After this time, the pin is automatically
    /// switched off.
    #[serde(default = "Gpio::max_on_time")]
    pub on_time: u64,
}

impl Gpio {
    pub fn max_on_time() -> u64 {
        u64::MAX
    }
}

/// Lambda heat pump Modbus data sink parameters.
#[derive(Clone, Debug, Deserialize)]
pub struct LambdaHeatPumpSink {
    /// Device IP address and port
    pub address: String,
}

/// Keba KeContact wallbox JSON data sink parameters.
#[derive(Clone, Debug, Deserialize)]
pub struct KeContactSink {
    /// Device IP address and port
    pub address: String,
}

/// Common type for handling different data sinks.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum SinkType {
    Debug,
    Gpio(Gpio),
    KeContact(KeContactSink),
    LambdaHeatPump(LambdaHeatPumpSink),
}

/// Defines a data sink node.
#[derive(Clone, Debug, Deserialize)]
pub struct Sink {
    pub name: String,
    #[serde(flatten)]
    pub variant: SinkType,
}

/// Overall settings for the empower-daemon.
#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Detach from controlling terminal after start.
    pub daemonize: bool,
    /// Test config and exit.
    pub test_cfg: bool,
    /// Path to PID file in daemon mode.
    pub pid_file: String,
    /// Path to the working directory in daemon mode.
    pub wrk_dir: String,
    /// Path to logfile directory.
    pub logfile: String,
    pub log_level: sloggers::types::Severity,
    pub database: Database,
    pub graphql: GraphQL,
    pub location: Option<Location>,

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
            test_cfg: false,
            pid_file: "/run/empowerd/pid".into(),
            wrk_dir: "/".into(),
            logfile: "/var/log/empowerd.log".into(),
            log_level: sloggers::types::Severity::Info,
            database: Database::default(),
            graphql: GraphQL::default(),
            location: None,
            sources: Vec::new(),
            processors: Vec::new(),
            sinks: Vec::new(),
        }
    }
}

impl Settings {
    /// Checks if a processor node is attached to the given data source.
    pub fn has_processor(&self, source: &str) -> bool {
        self.processors.iter().any({
            |x| match &x.variant {
                ProcessorType::Debug(x) => x.has_source(source),
                ProcessorType::AvailablePower(x) => x.has_source(source),
                ProcessorType::Appliance(x) => x.has_source(source),
                ProcessorType::LoadControl(x) => x.has_source(source),
            }
        })
    }

    /// Loads settings from config file.
    pub fn load_from_file(filename: &str) -> Result<Settings, String> {
        let toml = std::fs::read_to_string(filename).map_err(|e| {
            format!("Could not read config file '{}': {}", &filename, e)
        })?;
        return toml::from_str(&toml)
            .map_err(|e| format!("Could not parse config: {}", e));
    }

    /// Validates a node name.
    fn validate_name<'a>(
        names: &mut BTreeSet<&'a str>,
        name: &'a str,
    ) -> Result<(), String> {
        if names.contains(&name) {
            return Err(format!("Duplicate name in config file: '{}'", &name));
        } else if name.starts_with('_') {
            return Err(format!("Names must not start with '_': '{}'", &name));
        }
        names.insert(name);
        Ok(())
    }

    /// Validates complete settings.
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

    /// Loads settings from command line arguments and config file.
    pub fn load() -> Result<Settings, String> {
        let mut options = Options::new();
        options
            .optopt("c", "", "config filename", "NAME")
            .optflag("d", "", "nodaemonize")
            .optflag("t", "", "test config and exit");

        let matches = options.parse(env::args()).map_err(|e| e.to_string())?;

        let cfg_path = match matches.opt_str("c") {
            Some(x) => x,
            None => "/etc/empowerd/empowerd.conf".into(),
        };

        let mut settings = Settings::load_from_file(&cfg_path)?;

        if matches.opt_present("d") {
            settings.daemonize = false;
        }
        if matches.opt_present("t") {
            settings.daemonize = false;
            settings.test_cfg = true;
        }

        settings.validate()?;
        return Ok(settings);
    }
}
