use serde::Deserialize;
use std::env;

use config::{Config, ConfigError, File, FileFormat};
use getopts::Options;

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub daemonize: bool,
    pub pid_file: String,
    pub wrk_dir: String,
    pub listen_address: String,
    pub port: u16,
    pub session_timeout: u64,
    pub username: String,
    pub hashed_pw: String,
    pub pins: Vec<i64>,
}

impl Settings {
    fn load_from_file(filename: String) -> Result<Settings, ConfigError> {
        let mut config = Config::new();

        config.set_default("daemonize", false)?;
        config.set_default("pid_file", "/var/run/water/pid")?;
        config.set_default("wrk_dir", "/")?;
        config.set_default("listen_address", "127.0.0.1")?;
        config.set_default("port", 3000)?;
        config.set_default("session_timeout", 300)?;
        config.set_default("username", "water")?;
        config.set_default("hashed_pw", "!")?;
        config.set_default("pins", Vec::<i64>::new())?;

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

        let cfg_path = if matches.opt_present("c") {
            matches.opt_str("c").unwrap()
        } else {
            "/etc/water/water.conf".to_string()
        };

        let mut settings =
            Settings::load_from_file(cfg_path).map_err(|e| e.to_string())?;

        if matches.opt_present("d") {
            settings.daemonize = true;
        }

        return Ok(settings);
    }
}
