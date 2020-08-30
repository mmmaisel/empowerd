#![forbid(unsafe_code)]

use daemonize::Daemonize;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use juniper::RootNode;
use slog::{error, info, trace, Logger};
use std::convert::Infallible;
use std::net;
use std::process;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::signal;
use warp::Filter;

// TODO: implement logger (slog? + term)

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

    let term_decorator = slog_term::PlainSyncDecorator::new(std::io::stdout());
    let term_drain = slog_term::FullFormat::new(term_decorator).build();

    // TODO: async logging and filter level from config
    let root_logger = Logger::root(slog::Fuse(term_drain), slog::o!());
    info!(root_logger, "💦️ Starting waterd");
    trace!(root_logger, "Settings: {:?}", &settings);

    if settings.daemonize {
        let daemon = Daemonize::new()
            .pid_file(&settings.pid_file)
            .chown_pid_file(true)
            .working_directory(&settings.wrk_dir);

        match daemon.start() {
            Ok(_) => info!(root_logger, "Daemonized"),
            Err(e) => {
                error!(root_logger, "Daemonize failed: {}", e);
                process::exit(1);
            }
        }
    }

    match Runtime::new() {
        Ok(mut rt) => {
            process::exit(rt.block_on(tokio_main(settings, root_logger)))
        }
        Err(e) => {
            error!(root_logger, "Failed to create tokio runtime: {}", e);
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
    let water_switch = WaterSwitch::new(settings.pins);

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
