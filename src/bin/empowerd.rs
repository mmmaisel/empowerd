/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2024 Max Maisel

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
#![forbid(unsafe_code)]
#![allow(clippy::needless_return)]
#![allow(clippy::redundant_field_names)]
#![doc = include_str!("../../README.md")]

use daemonize::Daemonize;
use slog::{debug, error, info, trace, Logger};
use sloggers::{
    file::FileLoggerBuilder,
    terminal::{Destination, TerminalLoggerBuilder},
    types::OverflowStrategy,
    Build,
};

use std::{net, process, sync::Arc};
use tokio::{net::TcpListener, runtime::Runtime, signal};

use libempowerd::{
    graphql,
    processors::{self, ProcessorInfo},
    session_manager::SessionManager,
    settings::Settings,
    sinks, sources,
    task_group::TaskGroup,
    Globals,
};

fn main() {
    let settings = match Settings::load() {
        Ok(x) => x,
        Err(e) => {
            eprintln!("Could not load config: {}", e);
            process::exit(1);
        }
    };

    if settings.daemonize {
        let daemon = Daemonize::new()
            .pid_file(&settings.pid_file)
            .chown_pid_file(true)
            .working_directory(&settings.wrk_dir);

        if let Err(e) = daemon.start() {
            eprintln!("Daemonize failed: {}", e);
            process::exit(1);
        }
    }

    let root_logger = if settings.daemonize {
        FileLoggerBuilder::new(&settings.logfile)
            .level(settings.log_level)
            .overflow_strategy(OverflowStrategy::Block)
            .build()
    } else {
        TerminalLoggerBuilder::new()
            .level(settings.log_level)
            .overflow_strategy(OverflowStrategy::Block)
            .destination(Destination::Stdout)
            .build()
    };
    let root_logger = root_logger.unwrap();

    info!(root_logger, "⚡️ Starting empowerd");
    debug!(root_logger, "Settings: {:?}", &settings);

    let retval = match Runtime::new() {
        Ok(rt) => rt.block_on(tokio_main(settings, root_logger.clone())),
        Err(e) => {
            error!(root_logger, "Failed to create tokio runtime: {}", e);
            1
        }
    };
    trace!(root_logger, "Exiting with result {}", retval);
    drop(root_logger);
    process::exit(retval);
}

fn check_task_result(result: Result<(), ()>, logger: &Logger) -> i32 {
    match result {
        Ok(_) => {
            info!(logger, "Some task finished, exit.");
            0
        }
        Err(_) => {
            info!(logger, "Some task failed, exit.");
            1
        }
    }
}

async fn shutdown_group(
    mut group: TaskGroup,
    logger: &Logger,
) -> Result<(), ()> {
    if let Err(e) = group.cancel() {
        error!(logger, "Canceling {} failed: {}", group.name(), e);
        return Err(());
    }
    if group.run().await.is_err() {
        error!(logger, "Error occured during {} shutdown", group.name());
        return Err(());
    }
    Ok(())
}

async fn tokio_main(settings: Settings, logger: Logger) -> i32 {
    let (mut sources, outputs) =
        match sources::polling_tasks(logger.clone(), &settings) {
            Ok(x) => x,
            Err(e) => {
                error!(logger, "Initializing sources failed: {}", e);
                return 0;
            }
        };

    let (sinks, switch_proc_info) =
        match sinks::make_sinks(logger.clone(), &settings) {
            Ok(x) => x,
            Err(e) => {
                error!(logger, "Initializing sinks failed: {}", e);
                return 0;
            }
        };

    let switch_mux = match sinks.get("_SwitchMux") {
        Some(x) => match x {
            sinks::ArcSink::SwitchMux(x) => x.clone(),
            _ => {
                error!(logger, "SwitchMux has invalid type");
                return 2;
            }
        },
        None => {
            error!(logger, "Could not find SwitchMux sink");
            return 2;
        }
    };

    let ProcessorInfo {
        tasks: mut processors,
        commands: processor_cmds,
    } = match processors::processor_tasks(
        logger.clone(),
        &settings,
        outputs,
        sinks,
        switch_proc_info,
    ) {
        Ok(x) => x,
        Err(e) => {
            error!(logger, "Initializing processors failed: {}", e);
            return 0;
        }
    };

    let session_manager =
        match SessionManager::new(settings.graphql.session_timeout) {
            Ok(x) => x,
            Err(e) => {
                error!(logger, "Creating session manager failed: {}", e);
                return 2;
            }
        };

    let globals = Arc::new(Globals {
        logger: logger.clone(),
        username: settings.graphql.username.clone(),
        hashed_pw: settings.graphql.hashed_password.clone(),
        session_manager,
        switch_mux,
        processor_cmds,
    });

    let address =
        match settings.graphql.listen_address.parse::<net::SocketAddr>() {
            Ok(x) => x,
            Err(_) => {
                error!(
                    logger,
                    "{} is not an IP address", settings.graphql.listen_address
                );
                return 2;
            }
        };

    let listener = match TcpListener::bind(address).await {
        Ok(x) => x,
        Err(e) => {
            error!(logger, "Binding graphql socket failed: {e}");
            return 2;
        }
    };
    info!(logger, "Listening on http://{}", address);
    let server = tokio::task::spawn(graphql::server::run_graphql(
        listener,
        globals.clone(),
        logger.clone(),
    ));

    if settings.test_cfg {
        info!(logger, "Config valid");
        return 0;
    }

    let retval = tokio::select! {
        x = sources.run() => {
            check_task_result(x, &logger)
        }
        x = processors.run() => {
            check_task_result(x, &logger)
        }
        _ = server => {
            info!(logger, "server!!!");
            1
        }
        _ = signal::ctrl_c() => {
            info!(logger, "Received SIGINT, exit.");
            0
        }
    };

    let (source_result, processor_result) = tokio::join! {
        shutdown_group(sources, &logger),
        shutdown_group(processors, &logger),
    };
    if source_result.is_err() || processor_result.is_err() {
        return 2;
    }
    retval
}
