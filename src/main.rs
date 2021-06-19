#![forbid(unsafe_code)]

use daemonize::Daemonize;
use slog::{debug, error, info, trace, Logger};
use sloggers::{
    file::FileLoggerBuilder,
    terminal::{Destination, TerminalLoggerBuilder},
    types::{OverflowStrategy, Severity},
    Build,
};
use std::process;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::signal;
use hyper::{server::Server, Body, Method, Response, StatusCode, service::{make_service_fn, service_fn}};
use core::convert::Infallible;
use juniper::{EmptyMutation, EmptySubscription, RootNode};
use std::net;

mod miner;
mod models;
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

pub struct Context;
impl juniper::Context for Context {}
pub struct Field {
    hello: i32,
}

#[juniper::graphql_object(context = Context)]
impl Field {
    fn hello(&self) -> i32 {
        return 1;
    }
}

struct Query;

#[juniper::graphql_object(Context = Context)]
impl Query {
    async fn field() -> Field {
        Field {
            hello: 1,
        }
    }
}

async fn tokio_main(settings: Settings, logger: Logger) -> i32 {
    let address = "127.0.0.1:2345";//format!("{}:{}", settings.listen_address, settings.port);
    let address = match address.parse::<net::SocketAddr>() {
        Ok(x) => x,
        Err(_) => {
            error!(logger, "{} is not an IP address", address);
            return 2;
        }
    };

    // TODO: add graphql API and allow reconfigure settings over it
    // TODO: integrate graphql API server

    let root_node = Arc::new(RootNode::new(
        Query {},
        EmptyMutation::<Context>::new(),
        EmptySubscription::<Context>::new()
    ));

    let new_service = make_service_fn(move |_| {
        let root_node = root_node.clone();
        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let root_node = root_node.clone();
                //let ctx = ctx.clone();
                async {
                    Ok::<_, Infallible>(match (req.method(), req.uri().path()) {
                        (&Method::GET, "/") => juniper_hyper::graphiql("/graphql", None).await,
                        (&Method::GET, "/graphql") | (&Method::POST, "/graphql") => {
                            juniper_hyper::graphql(root_node, Arc::new(Context{}), req).await
                        }
                        _ => {
                            let mut response = Response::new(Body::empty());
                            *response.status_mut() = StatusCode::NOT_FOUND;
                            response
                        }
                    })
                }
            }))
        }
    });

    let server = Server::bind(&address).serve(new_service);
    info!(logger, "Listening on http://{}", address);

    let mut miner: Miner = match Miner::new(logger.clone(), &settings) {
        Ok(x) => x,
        Err(e) => {
            error!(logger, "Initializing miner failed: {}", e);
            return 0;
        }
    };

    let retval = tokio::select! {
        x = miner.run() => {
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

    if let Err(e) = miner.cancel() {
        error!(logger, "Canceling miner failed: {}", e);
        return 2;
    }
    if let Err(_) = miner.run().await {
        error!(logger, "Error occured during miner shutdown");
    }
    return retval;
}
