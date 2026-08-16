use axum::http::StatusCode;
use axum::response::{Response, IntoResponse};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    LoginFail
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("--> From implementation of error!");
        (StatusCode::UNAUTHORIZED, "User is not authorized or other error occured while logging in").into_response()
    }
}