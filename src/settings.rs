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
pub struct Battery {
    pub address: String,
    pub poll_interval: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Dachs {
    pub address: String,
    pub password: String,
    pub poll_interval: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Meter {
    pub device: String,
    pub baud: u32,
    pub poll_interval: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Solar {
    pub r#type: String,
    pub address: String,
    pub password: String,
    pub poll_interval: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Weather {
    pub poll_interval: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub daemonize: bool,
    pub pid_file: String,
    pub wrk_dir: String,
    pub logfile: String,
    pub log_level: u8,
    pub database: Database,

    pub battery: Option<Battery>,
    pub dachs: Option<Dachs>,
    pub meter: Option<Meter>,
    pub solar: Option<Solar>,
    pub weather: Option<Weather>,

/*
    pub listen_address: String,
    pub port: u16,
    pub session_timeout: u64,
    pub username: String,
    pub hashed_pw: String,
    pub pins: Vec<i64>,
    pub pin_names: Vec<String>,
*/
}

impl Settings {
    fn load_from_file(filename: String) -> Result<Settings, ConfigError> {
        let mut config = Config::new();

        config.set_default("daemonize", false)?;
        config.set_default("pid_file", "/run/stromd/pid")?;
        config.set_default("wrk_dir", "/")?;
        config.set_default("logfile", "/var/log/stromd.log")?;
        config.set_default("log_level", 0)?;
/*
        config.set_default("listen_address", "127.0.0.1")?;
        config.set_default("port", 3000)?;
        config.set_default("session_timeout", 300)?;
        config.set_default("username", "water")?;
        config.set_default("hashed_pw", "!")?;
        config.set_default("pins", Vec::<i64>::new())?;
        config.set_default("pin_names", Vec::<String>::new())?;
*/

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

        if let Some(settings) = &settings.meter {
            if settings.poll_interval < 5 {
                return Err("meter:poll_interval must be >= 5".into());
            }
        }
/*
        if settings.pins.len() != settings.pin_names.len() {
            return Err("'pins' and 'pin_names' must be of same size!".into());
        }
*/

        if matches.opt_present("d") {
            settings.daemonize = true;
        }

        return Ok(settings);
    }
}
