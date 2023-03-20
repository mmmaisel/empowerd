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
#![forbid(unsafe_code)]
#![allow(clippy::needless_return)]
#![allow(clippy::redundant_field_names)]

use daemonize::Daemonize;
use slog::{debug, error, info, trace, Logger};
use sloggers::{
    file::FileLoggerBuilder,
    terminal::{Destination, TerminalLoggerBuilder},
    types::OverflowStrategy,
    Build,
};

use hyper::{
    server::Server,
    service::{make_service_fn, service_fn},
    Body, Method, Response, StatusCode,
};
use juniper::{EmptySubscription, RootNode};
use std::{convert::Infallible, net, process, sync::Arc};
use tokio::{runtime::Runtime, signal};

mod graphql;
mod misc;
mod models;
mod multi_setpoint_hysteresis;
mod processors;
mod pt1;
mod session_manager;
mod settings;
mod sinks;
mod sources;
mod task_group;
mod tri_state;

use graphql::mutation::Mutation;
use graphql::query::Query;
use processors::{ProcessorCommands, ProcessorInfo};
use session_manager::SessionManager;
use settings::Settings;
use sinks::gpio_switch::GpioSwitch;
use task_group::TaskGroup;

#[derive(Debug)]
pub struct Globals {
    logger: Logger,
    username: String,
    hashed_pw: String,
    session_manager: SessionManager,
    gpio_switch: Arc<GpioSwitch>,
    processor_cmds: ProcessorCommands,
}

#[derive(Debug)]
pub struct Context {
    globals: Arc<Globals>,
    token: String,
}

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

    let (sinks, gpio_proc_info) =
        match sinks::make_sinks(logger.clone(), &settings) {
            Ok(x) => x,
            Err(e) => {
                error!(logger, "Initializing sinks failed: {}", e);
                return 0;
            }
        };

    let gpio_switch = match sinks.get("_GpioSwitch") {
        Some(x) => match x {
            sinks::ArcSink::GpioSwitch(x) => x.clone(),
            _ => {
                error!(logger, "GpioSwitch has invalid type");
                return 2;
            }
        },
        None => {
            error!(logger, "Could not find GpioSwitch sink");
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
        gpio_proc_info,
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
        gpio_switch,
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

    let root_node = Arc::new(RootNode::new(
        Query {},
        Mutation {},
        EmptySubscription::<Context>::new(),
    ));

    let new_service = make_service_fn(move |_| {
        let root_node = root_node.clone();
        let globals = globals.clone();
        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let root_node = root_node.clone();
                let globals = globals.clone();
                async {
                    Ok::<_, Infallible>(
                        match (req.method(), req.uri().path()) {
                            (&Method::GET, "/") => {
                                juniper_hyper::graphiql("/graphql", None).await
                            }
                            (&Method::GET, "/graphql")
                            | (&Method::POST, "/graphql") => {
                                let token =
                                    match req.headers().get("Authorization") {
                                        Some(x) => match x.to_str() {
                                            Ok(y) => y.replace("Bearer ", ""),
                                            Err(_) => "".into(),
                                        },
                                        None => "".into(),
                                    };
                                let context =
                                    Arc::new(Context { globals, token });
                                juniper_hyper::graphql(root_node, context, req)
                                    .await
                            }
                            _ => {
                                let mut response = Response::new(Body::empty());
                                *response.status_mut() = StatusCode::NOT_FOUND;
                                response
                            }
                        },
                    )
                }
            }))
        }
    });

    let server = Server::bind(&address).serve(new_service);
    info!(logger, "Listening on http://{}", address);

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
