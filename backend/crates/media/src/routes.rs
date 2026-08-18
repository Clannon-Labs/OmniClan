
use axum::{
    Router,
    routing::{
        post,
    },
};

use crate::upload::handle_media_upload;

pub fn media_routes() -> Router {
    Router::new()
        .route("/media/upload", post(handle_media_upload))
}
