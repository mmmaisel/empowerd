use serde::Deserialize;
use std::env;

use config::{Config, ConfigError, File, FileFormat};
use getopts::Options;

#[derive(Clone, Debug, Deserialize)]
pub struct Database {
    pub url: String,
    pub name: String,
    pub user: String,
    pub password: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SunnyIsland {
    pub name: String,
    pub address: String,
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
    SunnyIsland(SunnyIsland),
    DachsMsrS(DachsMsrS),
    SmlMeter(SmlMeter),
    SunnyBoySpeedwire(SunnyBoySpeedwire),
    Bresser6in1(Bresser6in1),
}

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub daemonize: bool,
    pub pid_file: String,
    pub wrk_dir: String,
    pub one_shot: bool,
    pub logfile: String,
    pub log_level: u8,
    pub database: Database,

    #[serde(rename = "source")]
    pub sources: Vec<Source>,
}

impl Settings {
    fn load_from_file(filename: String) -> Result<Settings, ConfigError> {
        let mut config = Config::new();

        config.set_default("daemonize", false)?;
        config.set_default("pid_file", "/run/empowerd/pid")?;
        config.set_default("wrk_dir", "/")?;
        config.set_default("one_shot", false)?;
        config.set_default("logfile", "/var/log/empowerd.log")?;
        config.set_default("log_level", 0)?;

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
            None => "/etc/empowerd/empowerd.conf".into(),
        };

        let mut settings =
            Settings::load_from_file(cfg_path).map_err(|e| e.to_string())?;

        if matches.opt_present("d") {
            settings.daemonize = true;
        }

        return Ok(settings);
    }
}
