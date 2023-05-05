#[macro_use]
extern crate dotenv_codegen;

use std::error::Error;

use async_graphql::{http::GraphiQLSource, EmptySubscription};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    response::{self, IntoResponse},
    routing::{get, post},
    Extension, Router, Server,
};
use tokio_postgres::NoTls;

mod models;
use models::{MutationRoot, QueryRoot, Schema};

async fn graphql_handler(schema: Extension<Schema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/").finish())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // connect to the database
    let (client, connection) = tokio_postgres::connect(
        dotenv!("DB_URL"),
        NoTls,
    )
    .await
    .unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // make the schema
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(client)
        .finish();

    // start the server
    let app = Router::new()
        .route("/", post(graphql_handler))
        .route("/graphiql", get(graphiql))
        .layer(Extension(schema));

    println!("GraphiQL: http://localhost:8000/graphiql");

    Server::bind(&"127.0.0.1:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
