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
        header,
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
    const MAX_UPLOAD_BYTES: u64 = 1024;
    
    let headers = request.headers();
    // Reject malformed or oversized declared lengths
    // before reading the body.
    match headers.get(header::CONTENT_LENGTH) {
        Some(length) => {
            match length.to_str(){
                Ok(len) => {
                    match len.parse::<u64>(){
                        Ok(parsed) if parsed > MAX_UPLOAD_BYTES  => {
                            let status_code = StatusCode::PAYLOAD_TOO_LARGE;
                            return (
                                status_code,
                                format!("{status_code}: Payload limit exceeded inside header after parsing!\n")
                            )
                        },
                        Err(err) => {
                            let status_code = StatusCode::BAD_REQUEST;
                            return (
                                status_code,
                                format!("{status_code}: Parsing failed with error: {err}\n")
                            )
                        },
                        Ok(_) => {}
                    }
                },
                Err(err) => {
                    let status_code = StatusCode::BAD_REQUEST;
                    return (
                        status_code,
                        format!("{status_code}: Conversion failed with error: {err}\n")
                    )
                },
            }
        }
        None => {}
    };
    
    // handler already owns the request;
    // into_body() consume it and returns its body
    let body = request.into_body();
    let mut stream = body.into_data_stream();
    
    let mut total_bytes: u64 = 0;

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {                
                match total_bytes.checked_add(chunk.len() as u64) {
                    Some(new_total) if new_total <= MAX_UPLOAD_BYTES => {
                            total_bytes = new_total
                        },
                    _ => {
                        let status_code = StatusCode::PAYLOAD_TOO_LARGE;
                        return (
                            status_code,
                            format!("{status_code}: Payload limit exceeded!\n")
                        ) 
                    }
                }
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
