
use axum::{
    Json,
    response::{
        Response, IntoResponse
    },
    http::StatusCode,
};
use serde_json::json;

use super::upload::UploadError;

#[derive(Debug)]
pub(crate) enum WorkerError {
    Io(std::io::Error),
    Upload(UploadError),
}

// Maybe writing during upload process lies in UploadError ?

// Internal functions always give their custom errors
// like UploadError etc but just incase std::io::Error comes
impl From<std::io::Error> for WorkerError {
    fn from(err: std::io::Error) -> Self {
        WorkerError::Io(err)
    }
}

impl From<UploadError> for WorkerError {
    fn from(err: UploadError) -> Self {
        Self::Upload(err)
    }
}

impl std::fmt::Display for WorkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "Worker I/O error; {err}"),
            Self::Upload(err) => write!(f, "Upload error: {err}"),
        }
    }
}

impl std::error::Error for WorkerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Upload(err) => Some(err),
        }
    }
}

impl IntoResponse for WorkerError {
    fn into_response(self) -> Response {
        match self {
            Self::Io {..} => {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(
                        json!({
                            "error": "Internal server error"
                        })
                    )
                    ).into_response()
            },
            Self::Upload(err) => match err {
                UploadError::Io {..} => {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(
                            json!({
                                "error": "Internal server error"
                            })
                        )
                        ).into_response()
                },
                UploadError::TooLarge{..} => {
                    (
                        StatusCode::PAYLOAD_TOO_LARGE,
                        Json(
                            json!({
                                "error": "Given media exceeded size limit!"
                            })
                        )
                        ).into_response()
                },
                UploadError::InvalidMedia{..} => {
                    (
                        StatusCode::UNSUPPORTED_MEDIA_TYPE,
                        Json(json!({
                            "error": "Given media is either invalid or not supported!"
                        }))
                    ).into_response()
                },
                UploadError::InvalidState{..} => {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "error": "Given media is in invalid state!"
                        }))
                    ).into_response()
                },
                UploadError::Write{..} => {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "error": "Failed to write media to disk!"
                        }))
                    ).into_response()
                },
            }
        }
    }
}
