#![deny(warnings)]

use crate::database::models::internal_user::InternalUser;
use crate::database::{DatabaseConfig, PgPool};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use ring::pbkdf2::PBKDF2_HMAC_SHA512;
use ring::{digest, pbkdf2};
use std::num::NonZeroU32;
use std::sync::Arc;
use warp::Rejection;
use warp::{self, Filter};

const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;
pub type Credential = [u8; CREDENTIAL_LEN];

#[derive(Clone)]
pub struct Session {
    pub connection_pool: PgPool,
}

pub fn with_session(
    session: Session,
) -> impl Filter<Extract = (Session,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || session.clone())
}

pub fn with_predicate(
    predicate: fn(InternalUser) -> Result<InternalUser, Rejection>,
) -> impl Filter<
    Extract = (fn(InternalUser) -> Result<InternalUser, Rejection>,),
    Error = std::convert::Infallible,
> + Clone {
    warp::any().map(move || predicate)
}

pub fn with_db_config(
    db_config: Arc<DatabaseConfig>,
) -> impl Filter<Extract = (Arc<DatabaseConfig>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db_config.clone())
}

pub fn random_string(n: usize) -> String {
    thread_rng().sample_iter(&Alphanumeric).take(n).collect()
}

pub fn hash_password(password: String, salt: String, iterations: NonZeroU32) -> Credential {
    let mut result: Credential = [0u8; CREDENTIAL_LEN];
    pbkdf2::derive(
        PBKDF2_HMAC_SHA512,
        iterations,
        salt.as_bytes(),
        password.as_bytes(),
        &mut result,
    );
    result
}
