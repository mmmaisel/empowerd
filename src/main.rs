#![forbid(unsafe_code)]

use daemonize::Daemonize;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use juniper::RootNode;
use std::convert::Infallible;
use std::net;
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
    eprintln!("Raw token: {:?}", &ctx.token);

    let res = req.execute(&schema, &ctx); //.await;
    return Ok(serde_json::to_string(&res).expect("Invalid JSON response"));
}

fn main() {
    let settings = match Settings::load() {
        Ok(x) => x,
        Err(e) => panic!("Could not load config: {}", e),
    };

    if settings.daemonize {
        let daemon = Daemonize::new()
            .pid_file(&settings.pid_file)
            .chown_pid_file(true)
            .working_directory(&settings.wrk_dir);

        match daemon.start() {
            Ok(_) => println!("Daemonized"),
            Err(e) => panic!("Daemonize failed: {}", e),
        }
    }

    match Runtime::new() {
        Ok(mut rt) => rt.block_on(tokio_main(settings)),
        Err(e) => panic!("Failed to create tokio runtime: {}", e),
    };
}

async fn tokio_main(settings: Settings) {
    let schema = Arc::new(Schema::new(Query, Mutation));
    let schema = warp::any().map(move || Arc::clone(&schema));

    let session_manager = match SessionManager::new(settings.session_timeout) {
        Ok(x) => x,
        Err(e) => panic!("Creating session manager failed: {}", e),
    };
    let water_switch = WaterSwitch::new(settings.pins);

    let globals = Arc::new(Globals {
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
        Err(_) => panic!("{} is not an IP address", address),
    };

    println!("Ready");
    tokio::select! {
        _ = warp::serve(routes).run(address) => {
            println!("Server loop exited.");
        }
        _ = signal::ctrl_c() => {
            println!("Received SIGINT, exit.");
        }
    }
}
