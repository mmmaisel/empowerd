/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2021 Max Maisel

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
    types::{OverflowStrategy, Severity},
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
mod models;
mod settings;
mod sources;

mod gpio_switch;
mod session_manager;

use settings::{Settings, Sink};
use sources::Sources;

use gpio_switch::GpioSwitch;
use graphql::mutation::Mutation;
use graphql::query::Query;
use session_manager::SessionManager;

#[derive(Debug)]
pub struct Globals {
    logger: Logger,
    username: String,
    hashed_pw: String,
    session_manager: SessionManager,
    gpio_switch: GpioSwitch,
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

async fn tokio_main(settings: Settings, logger: Logger) -> i32 {
    let session_manager =
        match SessionManager::new(settings.graphql.session_timeout) {
            Ok(x) => x,
            Err(e) => {
                error!(logger, "Creating session manager failed: {}", e);
                return 2;
            }
        };

    let gpios = settings
        .sinks
        .clone()
        .into_iter()
        .filter_map(|sink| match sink {
            Sink::Gpio(gpio) => Some(gpio),
            _ => None,
        })
        .collect();
    let gpio_switch = match GpioSwitch::new(gpios) {
        Ok(x) => x,
        Err(e) => {
            error!(logger, "Could not create water switch: {}", e);
            return 2;
        }
    };

    let globals = Arc::new(Globals {
        logger: logger.clone(),
        username: settings.graphql.username.clone(),
        hashed_pw: settings.graphql.hashed_password.clone(),
        session_manager,
        gpio_switch,
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

    let mut sources: Sources = match Sources::new(logger.clone(), &settings) {
        Ok(x) => x,
        Err(e) => {
            error!(logger, "Initializing sources failed: {}", e);
            return 0;
        }
    };

    let retval = tokio::select! {
        x = sources.run() => {
            match x {
                Ok(_) => {
                    info!(logger, "Some task finished, exit.");
                    0
                },
                Err(_) => {
                    info!(logger, "Some task failed, exit.");
                    1
                }
            }
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

    if let Err(e) = sources.cancel() {
        error!(logger, "Canceling sources failed: {}", e);
        return 2;
    }
    if sources.run().await.is_err() {
        error!(logger, "Error occured during sources shutdown");
    }
    return retval;
}
