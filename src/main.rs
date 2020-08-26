use daemonize::Daemonize;
use juniper::http::graphiql::graphiql_source;
use juniper::RootNode;
use std::net;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::signal;
use warp::Filter;

// TODO: implement logger (slog?)

mod mutation;
mod query;
mod settings;
mod valve;
mod water_switch;

use mutation::*;
use query::*;
use settings::*;
use valve::*;
use water_switch::*;

type Schema = RootNode<'static, Query, Mutation>;

pub struct Context {
    water_switch: WaterSwitch,
}

use juniper::http::GraphQLRequest;
use std::convert::Infallible;

async fn graphql(
    schema: Arc<Schema>,
    ctx: Arc<Context>,
    req: GraphQLRequest,
) -> Result<impl warp::Reply, Infallible> {
    let res = req.execute(&schema, &ctx); //.await;
    let json = serde_json::to_string(&res).expect("Invalid JSON response");
    Ok(json)
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

    let water_switch = WaterSwitch::new(settings.pins);

    let ctx = Arc::new(Context {
        water_switch: water_switch,
    });
    let ctx = warp::any().map(move || Arc::clone(&ctx));

    let graphql_route = warp::post()
        .and(warp::path!("graphql"))
        .and(schema.clone())
        .and(ctx.clone())
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
