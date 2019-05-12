use std::env;

extern crate config;
use config::{Config, File, FileFormat};

// https://github.com/mehcode/config-rs/blob/master/examples/hierarchical-env/src/settings.rs
#[derive(Debug, Deserialize)]
pub struct Settings
{
    pub daemonize: bool,
    pub dachs_addr: String,
    pub dachs_pw: String,
    pub sma_addr: String,
    pub sma_pw: String,
    pub db_url: String,
    pub db_pw: String,
    // TODO: polling should be 300s aligned
    pub dachs_poll_interval: u32,
    pub sma_poll_interval: u32
}

impl Settings
{
    pub fn load_config(filename: String) -> Settings
    {
        let mut config = Config::new();

        config.set_default("daemonize", true);
        config.set_default("dachs_poll_interval", 300);
        config.set_default("sma_poll_interval", 3600);

        config.merge(File::with_name(&filename).
            format(FileFormat::Hjson)).expect("🧠️");

        let mut settings: Settings = config.try_into().expect("🧠️");

        for arg in env::args()
        {
            // TODO: add nodaemonize
            if arg == "-d"
            {
                settings.daemonize = true;
            }
        }

        return settings;
    }
}
