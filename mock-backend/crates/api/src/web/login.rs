use crate::{Error, Result};
use axum::{
    {Router, Json},
    {routing::post},
};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};

#[derive(Debug, Serialize, Deserialize)]
struct LoginPayload {
    pub username: String,
    pub password: String,
}

pub fn login_route() -> Router {
    Router::new()
        .route("/login", post(login))
}

async fn login(payload:Json<LoginPayload>) -> Result<Json<Value>> {
    println!("--> LOGIN HANDLER --> We are inside login handler!");

    // TODO: Implement real database and auth logic here
    if payload.username != "cybro" || payload.password != "cybro123"{
        return Err(Error::LoginFail)
    }

    // TODO: Set cookies so client can keep the login alive

    // Create a proper response body
    let body = Json(json!({
        "result": {
            "success": true
        }
    }));

    Ok(body)
}
