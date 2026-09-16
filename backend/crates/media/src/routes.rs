
use axum::{
    Router,
    routing::post,
};

use super::workers::upload_worker;

pub fn handle_routes() -> Router {
    Router::new()
        .route("/media/upload", post(upload_worker))
}
