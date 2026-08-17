use axum::{
    Router,
    extract::{
        Request,
    },
    response::{Html, IntoResponse},
    routing::{
        get,
        post,
    },
    http::{
        StatusCode,
    }
};
use futures_util::StreamExt;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(handle_health))
        .route("/media", post(handle_media))
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

async fn handle_media(request: Request) -> (StatusCode, String) {
    // consume original request and take ownership of it
    // which makes original request unavailable afterwards
    let body = request.into_body();
    let mut stream = body.into_data_stream();

    let mut total_bytes: u64 = 0;

    while let Some(chunk_result) = stream.next().await{
        match chunk_result {
            Ok(chunk) => {
                total_bytes += chunk.len() as u64
            }
            Err(err) => {
                return (
                    StatusCode::BAD_REQUEST,
                    format!("Request body couldn't be read: {err}\n"),
                );
            }
        }
    }
    (
        StatusCode::OK,
        format!("Received {total_bytes} bytes\n")
    )
}
