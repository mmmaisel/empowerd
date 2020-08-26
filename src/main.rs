use juniper::http::graphiql::graphiql_source;
use juniper::RootNode;
use std::net;
use std::sync::Arc;
use tokio::signal;
use warp::Filter;

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

#[tokio::main]
async fn main() {
    let schema = Arc::new(Schema::new(Query, Mutation));
    let schema = warp::any().map(move || Arc::clone(&schema));

    let settings = match Settings::load() {
        Ok(x) => x,
        Err(e) => panic!("Could not load config: {}", e),
    };

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
