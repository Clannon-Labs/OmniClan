use axum::{
    Router,
    response::{Html, IntoResponse},
    routing::{
        get,
    },
    http::{
        StatusCode,
    }
};
use media;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(handle_health))
        .merge(media::handle_routes())
        .fallback(handle_not_found);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Failure occured while binding to a port");

    println!(
        "Listening on {}",
        listener.local_addr().expect("Can't find the local address")
    );

    axum::serve(listener, app)
        .await
        .expect("Couldn't serve the listener and app");
}

async fn handle_not_found() -> (StatusCode, Html<&'static str>) {
    (
        StatusCode::NOT_FOUND,
        Html(include_str!("../services/fallback.html"))
    )
}

async fn handle_health() -> impl IntoResponse {
    Html("It's healthy!\n")
}
