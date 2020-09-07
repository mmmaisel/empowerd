use std::env;

extern crate config;
use config::{Config, ConfigError, File, FileFormat};

// https://github.com/mehcode/config-rs/blob/master/examples/hierarchical-env/src/settings.rs
#[derive(Clone, Debug, Deserialize)]
pub struct Settings
{
    pub daemonize: bool,
    pub pid_file: String,
    pub wrk_dir: String,
    pub one_shot: bool,
    pub log_filename: String,
    pub log_level: u8,
    pub dachs_addr: String,
    pub dachs_pw: String,
    pub sma_addr: String,
    pub sma_pw: String,
    pub meter_device: String,
    pub meter_baud: u32,
    pub battery_addr: String,
    pub db_url: String,
    pub db_name: String,
    pub db_user: String,
    pub db_pw: String,
    // TODO: polling should be 300s aligned
    pub dachs_poll_interval: i64,
    pub sma_poll_interval: i64,
    pub battery_poll_interval: i64,
    pub meter_poll_interval: i64,
    pub weather_poll_interval: i64,
    pub import: String
}

impl Settings
{
    pub fn load_config(filename: String)
        -> Result<Settings, ConfigError>
    {
        let mut config = Config::new();

        config.set_default("daemonize", true)?;
        config.set_default("pid_file", "/var/run/stromd/pid")?;
        config.set_default("wrk_dir", "/")?;
        config.set_default("one_shot", false)?;
        config.set_default("log_filename", "/var/log/stromd.log")?;
        config.set_default("log_level", 5)?;
        config.set_default("dachs_poll_interval", 300)?;
        config.set_default("sma_poll_interval", 3600)?;
        config.set_default("battery_poll_interval", 300)?;
        config.set_default("meter_poll_interval", 300)?;
        config.set_default("weather_poll_interval", 300)?;
        config.set_default("import", "none")?;

        config.merge(File::with_name(&filename).
            format(FileFormat::Hjson))?;

        let mut settings: Settings = config.try_into()?;

        if settings.meter_poll_interval < 5
        {
            return Err(ConfigError::Message(
                "meter_poll_interval must be >= 5".to_string()));
        }

        let mut found_import = false;
        for arg in env::args()
        {
            // TODO: add nodaemonize
            if arg == "-d"
            {
                settings.daemonize = true;
            }

            if arg == "--import"
            {
                found_import = true;
            }
            else if found_import == true
            {
                settings.import = arg;
                found_import = false;
            }
        }
        return Ok(settings);
    }
}
