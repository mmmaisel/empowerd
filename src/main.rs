use warp::Filter;
use std::sync::Arc;
use juniper::http::graphiql::graphiql_source;
use juniper::RootNode;

#[derive(juniper::GraphQLObject)]
struct ValveState {
    state: i32,
}

struct QueryRoot;
struct MutationRoot;

#[juniper::object(Context = Context)]
impl QueryRoot {
    async fn valves(ctx: &Context) -> juniper::FieldResult<ValveState> {
        return Ok(ValveState { state: 123 });
    }
}

#[juniper::object(Context = Context)]
impl MutationRoot {
    async fn set_valves(ctx: &Context, state: i32) -> juniper::FieldResult<ValveState> {
        return Ok(ValveState { state: state });
    }
}

type Schema = RootNode<'static, QueryRoot, MutationRoot>;

struct Context {
}

use std::convert::Infallible;
use juniper::http::GraphQLRequest;

async fn graphql(
    schema: Arc<Schema>,
    ctx: Arc<Context>,
    req: GraphQLRequest,
) -> Result<impl warp::Reply, Infallible> {
    let res = req.execute(&schema, &ctx);//.await;
    let json = serde_json::to_string(&res).expect("Invalid JSON response");
    Ok(json)
}

#[tokio::main]
async fn main () {
    let schema = Arc::new(Schema::new(QueryRoot, MutationRoot));
    let schema = warp::any().map(move || Arc::clone(&schema));

    let ctx = Arc::new(Context { });
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

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}
