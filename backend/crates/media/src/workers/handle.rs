
use axum::{
    Json,
    response::IntoResponse,
    http::{
        HeaderMap,
    },
    body::Body,
};

use super::upload::{
    handle_upload,

    UploadRequest,
};
use super::WorkerError;

pub(crate) async fn upload_worker(
    headers: HeaderMap,
    body: Body,
) -> Result<impl IntoResponse, WorkerError> {
    let media = handle_upload(
        UploadRequest {
            headers,
            body
        }
    ).await?;

    // Don't expose the whole FinalMedia's data
    // But it's okay for until we make it work first
    Ok(
        Json(media)
    )
}