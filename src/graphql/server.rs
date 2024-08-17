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

use super::{mutation::Mutation, query::Query};
use crate::{Context, Globals};
use hyper::{
    body::Incoming, server::conn::http1, service::service_fn, Method, Request,
    Response, StatusCode,
};
use hyper_util::rt::TokioIo;
use juniper::{EmptySubscription, RootNode};
use slog::{error, Logger};
use std::{convert::Infallible, sync::Arc};
use tokio::net::TcpListener;

async fn handle_connection(
    req: Request<Incoming>,
    root_node: Arc<
        RootNode<'static, Query, Mutation, EmptySubscription<Context>>,
    >,
    globals: Arc<Globals>,
) -> Result<Response<String>, Infallible> {
    Ok::<_, Infallible>(match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => juniper_hyper::graphiql("/graphql", None).await,
        (&Method::GET, "/graphql") | (&Method::POST, "/graphql") => {
            let token = match req.headers().get("Authorization") {
                Some(x) => match x.to_str() {
                    Ok(y) => y.replace("Bearer ", ""),
                    Err(_) => "".into(),
                },
                None => "".into(),
            };
            let context = Arc::new(Context { globals, token });
            juniper_hyper::graphql(root_node, context, req).await
        }
        _ => {
            let mut response = Response::new(String::new());
            *response.status_mut() = StatusCode::NOT_FOUND;
            response
        }
    })
}

pub async fn run_graphql(
    listener: TcpListener,
    globals: Arc<Globals>,
    logger: Logger,
) -> Result<(), std::io::Error> {
    let root_node = Arc::new(RootNode::new(
        Query {},
        Mutation {},
        EmptySubscription::<Context>::new(),
    ));

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let root_node = root_node.clone();
        let globals = globals.clone();
        let logger = logger.clone();

        tokio::spawn(async move {
            let conn_result = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| {
                        handle_connection(
                            req,
                            root_node.clone(),
                            globals.clone(),
                        )
                    }),
                )
                .await;

            if let Err(e) = conn_result {
                error!(logger, "Handling connection failed: {e}");
            };
        });
    }
}
