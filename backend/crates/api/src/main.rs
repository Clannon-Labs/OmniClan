#![allow(unused)]

use axum::{
    Router,
    {routing::get},
    {response::Html},
};

#[tokio::main]
async fn main() {
    let greet = Router::new().route(
        "/greet",
        get(||async {Html("Hey buddy!")}),
    );

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
