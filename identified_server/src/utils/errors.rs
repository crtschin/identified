#![deny(warnings)]

use serde::Serialize;
use std::convert::Infallible;
use warp::{http::StatusCode, reject, Rejection, Reply};

impl reject::Reject for DatabaseConnectionError {}
impl reject::Reject for ValidationError {}
impl reject::Reject for InputError {}
impl reject::Reject for AuthenticationError {}
impl reject::Reject for AuthorizationError {}

#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

pub fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    } else if let Some(AuthorizationError::NoToken) = err.find() {
        code = StatusCode::UNAUTHORIZED;
        message = "INVALID TOKEN";
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        // We can handle a specific error, here METHOD_NOT_ALLOWED,
        // and render it however we want
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED";
    } else {
        // We should have expected this... Just log and say its a 500
        eprintln!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION";
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}

#[derive(Serialize, Debug)]
pub enum Error {
    Db(DatabaseConnectionError),
    Input(InputError),
    Authentication(AuthenticationError),
    Authorization(AuthorizationError),
}

#[derive(Serialize, Debug)]
pub struct DatabaseConnectionError {
    pub msg: String,
}

#[derive(Serialize, Debug)]
pub enum ValidationError {
    Required,
    AlreadyExists,
}

#[derive(Serialize, Debug)]
pub struct InputError {
    pub fields: (String, ValidationError),
}

#[derive(Serialize, Debug)]
pub enum AuthenticationError {
    InvalidLogin,
}

#[derive(Serialize, Debug)]
pub enum AuthorizationError {
    Unauthorized,
    ExpiredToken,
    InvalidToken,
    NoToken,
}
