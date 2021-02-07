#![forbid(unsafe_code)]

use daemonize::Daemonize;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use juniper::RootNode;
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
use warp::Filter;

mod mutation;
mod query;
mod session_manager;
mod settings;
mod valve;
mod water_switch;

use mutation::*;
use query::*;
use session_manager::*;
use settings::*;
use valve::*;
use water_switch::*;

type Schema = RootNode<'static, Query, Mutation>;

pub struct Globals {
    logger: Logger,
    username: String,
    hashed_pw: String,
    session_manager: SessionManager,
    water_switch: WaterSwitch,
}

pub struct Context {
    globals: Arc<Globals>,
    token: String,
}

async fn graphql(
    auth: Option<String>,
    schema: Arc<Schema>,
    globals: Arc<Globals>,
    req: GraphQLRequest,
) -> Result<impl warp::Reply, Infallible> {
    let ctx = Context {
        globals: globals.clone(),
        token: auth.unwrap_or("".into()).replace("Bearer ", ""),
    };

    let res = req.execute(&schema, &ctx); //.await;
    return Ok(serde_json::to_string(&res).expect("Invalid JSON response"));
}

fn main() {
    let settings = match Settings::load() {
        Ok(x) => x,
        Err(e) => {
            eprintln!("Could not load config: {}", e);
            process::exit(1);
        }
    };

    let root_logger = if settings.daemonize {
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

    info!(root_logger, "💦️ Starting waterd");
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
    let schema = Arc::new(Schema::new(Query, Mutation));
    let schema = warp::any().map(move || Arc::clone(&schema));

    let session_manager = match SessionManager::new(settings.session_timeout) {
        Ok(x) => x,
        Err(e) => {
            error!(logger, "Creating session manager failed: {}", e);
            return 2;
        }
    };
    let water_switch = match WaterSwitch::new(settings.pins, settings.pin_names)
    {
        Ok(x) => x,
        Err(e) => {
            error!(logger, "Could not create water switch: {}", e);
            return 2;
        }
    };

    let globals = Arc::new(Globals {
        logger: logger.clone(),
        username: settings.username,
        hashed_pw: settings.hashed_pw,
        session_manager: session_manager,
        water_switch: water_switch,
    });
    let globals = warp::any().map(move || Arc::clone(&globals));

    let graphql_route = warp::post()
        .and(warp::path!("graphql"))
        .and(warp::header::optional("Authorization"))
        .and(schema.clone())
        .and(globals.clone())
        .and(warp::body::json())
        .and_then(graphql);

    let graphiql_route = warp::get()
        .and(warp::path!("graphiql"))
        .map(|| warp::reply::html(graphiql_source("graphql")));

    let routes = graphql_route.or(graphiql_route);
    let address = format!("{}:{}", settings.listen_address, settings.port);
    let address = match address.parse::<net::SocketAddr>() {
        Ok(x) => x,
        Err(_) => {
            error!(logger, "{} is not an IP address", address);
            return 2;
        }
    };

    info!(
        logger,
        "Listening on {}:{}", settings.listen_address, settings.port
    );
    tokio::select! {
        _ = warp::serve(routes).run(address) => {
            error!(logger, "Server loop has terminated!");
            return 3;
        }
        _ = signal::ctrl_c() => {
            info!(logger, "Received SIGINT, exit.");
            return 0;
        }
    }
}
