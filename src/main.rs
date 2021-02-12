#![forbid(unsafe_code)]

use daemonize::Daemonize;
use slog::{debug, error, info, Logger};
use sloggers::{
    file::FileLoggerBuilder,
    terminal::{Destination, TerminalLoggerBuilder},
    types::{OverflowStrategy, Severity},
    Build,
};
use std::convert::Infallible;
use std::net;
use std::process;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::signal;

mod miner;
mod settings;

use miner::Miner;
use settings::Settings;

fn main() {
    let settings = match Settings::load() {
        Ok(x) => x,
        Err(e) => {
            eprintln!("Could not load config: {}", e);
            process::exit(1);
        }
    };

    let root_logger = if settings.daemonize {
        // TODO: evaluate log level here
        FileLoggerBuilder::new(&settings.logfile)
            .level(Severity::Info)
            .overflow_strategy(OverflowStrategy::Block)
            .build()
    } else {
        TerminalLoggerBuilder::new()
            .level(Severity::Trace)
            .overflow_strategy(OverflowStrategy::Block)
            .destination(Destination::Stdout)
            .build()
    };
    let root_logger = root_logger.unwrap();

    info!(root_logger, "⚡️ Starting stromd");
    debug!(root_logger, "Settings: {:?}", &settings);

    if settings.daemonize {
        let daemon = Daemonize::new()
            .pid_file(&settings.pid_file)
            .chown_pid_file(true)
            .working_directory(&settings.wrk_dir);

        match daemon.start() {
            Ok(_) => info!(root_logger, "Daemonized"),
            Err(e) => {
                error!(root_logger, "Daemonize failed: {}", e);
                drop(root_logger);
                process::exit(1);
            }
        }
    }

    match Runtime::new() {
        Ok(rt) => {
            let retval = rt.block_on(tokio_main(settings, root_logger.clone()));
            drop(root_logger);
            process::exit(retval);
        }
        Err(e) => {
            error!(root_logger, "Failed to create tokio runtime: {}", e);
            drop(root_logger);
            process::exit(1);
        }
    };
}

async fn tokio_main(settings: Settings, logger: Logger) -> i32 {
    /*    let address = format!("{}:{}", settings.listen_address, settings.port);
    let address = match address.parse::<net::SocketAddr>() {
        Ok(x) => x,
        Err(_) => {
            error!(logger, "{} is not an IP address", address);
            return 2;
        }
    };*/

    let mut miner: Miner = match Miner::new(logger.clone(), &settings) {
        Ok(x) => x,
        Err(e) => {
            error!(logger, "Initializing miner failed: {}", e);
            return 0;
        }
    };

    tokio::select! {
        _ = miner.run() => {
            info!(logger, "Task X failed, exit.");
            //return 0;
        }
        _ = signal::ctrl_c() => {
            info!(logger, "Received SIGINT, exit.");
            //return 0;
        }
    }

    info!(logger, "EXIT");
    miner.run().await;
    // TODO: now set cancel bit and join all again
    return 0;
}
