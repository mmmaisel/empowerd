use serde::Deserialize;
use std::env;

use config::{Config, ConfigError, File, FileFormat};
use getopts::Options;

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub daemonize: bool,
    pub pid_file: String,
    pub wrk_dir: String,
    pub one_shot: bool,
    pub logfile: String,
    pub log_level: u8,
    pub db_url: String,
    pub db_name: String,
    pub db_user: String,
    pub db_pw: String,

    pub enable_battery: bool,
    pub battery_addr: String,
    pub battery_poll_interval: u64,

    pub enable_dachs: bool,
    pub dachs_addr: String,
    pub dachs_pw: String,
    pub dachs_poll_interval: u64,

    pub enable_meter: bool,
    pub meter_device: String,
    pub meter_baud: u32,
    pub meter_poll_interval: u64,

    pub enable_solar: bool,
    pub solar_type: String,
    pub solar_addr: String,
    pub solar_pw: String,
    pub solar_poll_interval: u64,

    pub enable_weather: bool,
    pub weather_poll_interval: u64,
}

impl Settings {
    fn load_from_file(filename: String) -> Result<Settings, ConfigError> {
        let mut config = Config::new();

        config.set_default("daemonize", false)?;
        config.set_default("pid_file", "/run/stromd/pid")?;
        config.set_default("wrk_dir", "/")?;
        config.set_default("one_shot", false)?;
        config.set_default("logfile", "/var/log/stromd.log")?;
        config.set_default("log_level", 0)?;

        config.set_default("db_url", "127.0.0.1:8086")?;
        config.set_default("db_name", "stromd")?;
        config.set_default("db_user", "stromd")?;

        config.set_default("enable_battery", false)?;
        config.set_default("battery_poll_interval", 300)?;

        config.set_default("enable_dachs", false)?;
        config.set_default("dachs_poll_interval", 300)?;

        config.set_default("enable_meter", false)?;
        config.set_default("meter_poll_interval", 300)?;

        config.set_default("enable_solar", false)?;
        config.set_default("solar_poll_interval", 3600)?;

        config.set_default("enable_weather", false)?;
        config.set_default("weather_poll_interval", 300)?;

        config.merge(File::with_name(&filename).format(FileFormat::Toml))?;

        return config.try_into();
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
            None => "/etc/stromd/stromd.conf".into(),
        };

        let mut settings =
            Settings::load_from_file(cfg_path).map_err(|e| e.to_string())?;

        if settings.meter_poll_interval < 5 {
            return Err("meter_poll_interval must be >= 5".into());
        }

        if matches.opt_present("d") {
            settings.daemonize = true;
        }

        return Ok(settings);
    }
}
