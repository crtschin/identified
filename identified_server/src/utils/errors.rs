use serde::Serialize;
use warp::reject;

impl reject::Reject for Error {}
impl reject::Reject for DbError {}
impl reject::Reject for ValidationError {}
impl reject::Reject for InputError {}
impl reject::Reject for AuthenticationError {}
impl reject::Reject for AuthorizationError {}
impl reject::Reject for ServerError {}

#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

#[derive(Serialize, Debug)]
pub enum Error {
    Server(ServerError),
    Db(DbError),
    Input(InputError),
    Authentication(AuthenticationError),
    Authorization(AuthorizationError),
}

#[derive(Serialize, Debug)]
pub enum ServerError {
    SerializationError(String),
}

#[derive(Serialize, Debug)]
pub enum DbError {
    DatabaseConnectionError(String),
    DatabaseQueryError(String),
}

#[derive(Serialize, Debug)]
pub enum ValidationError {
    Required,
    AlreadyExists,
}

#[derive(Serialize, Debug)]
pub struct InputError {
    pub field: (String, ValidationError),
}

#[derive(Serialize, Debug)]
pub enum AuthenticationError {
    InvalidEmail,
    InvalidPassword,
    CouldNotGenerateAuthToken,
    InvalidLogin(String),
}

#[derive(Serialize, Debug)]
pub enum AuthorizationError {
    Unauthorized,
    ExpiredToken,
    InvalidToken,
    NoToken,
}
