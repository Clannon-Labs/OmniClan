#![allow(unused)]

use axum::{
    Router,
    {extract::{
        Query,
        Path,
    }},
    {routing::get},
    {response::{
        Html,
        IntoResponse
    }},
};
use serde::Deserialize;
use serde_json;

#[derive(Debug, Deserialize)]
struct GreetParams {
    name: Option<String>,
    message: Option<String>,
}

#[tokio::main]
async fn main() {
    let greet = Router::new()
        .route("/greet",get(handle_greet))
        .route("/greet2/{name}", get(handle_greet_with_name));

    // start of server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    println!("Listening on 127.0.0.1:8080");

    axum::serve(listener, greet)
        .await
        .unwrap();
    // end of server
}

async fn handle_greet(Query(params): Query<GreetParams>) -> impl IntoResponse {
    println!("->> {:<12} - Inside greet handler - {params:?}", "HANDLER");

    let name = params.name.as_deref().unwrap_or("Stranger");
    let message = params.message.as_deref().unwrap_or("Hello, ");
    
    Html(format!("{message}<strong>{name}<strong>"))
}

async fn handle_greet_with_name(Path(name): Path<String>) -> impl IntoResponse {
    println!("-->> {:<12} - Inside greet handler 2 - {name:?}", "HANDLER");

    Html(format!("I am using your name '{name}' to greet you"))
}
