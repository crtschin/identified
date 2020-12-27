use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use warp::{path::end, Filter, Rejection, Reply};

use crate::{
    api::helpers::authorization::*,
    api::internal::filters::main_filter as internal_filter,
    database::models::internal_user::InternalUser,
    database::{get_connection, DatabaseConfig},
    utils::common::*,
};

pub mod filters {
    use super::*;

    pub fn main_filter(
        db_config: Arc<DatabaseConfig>,
        session: Arc<Session>,
    ) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        let login = warp::path("login").and(login_filter(db_config.clone(), session.clone()));
        let internal = warp::path("internal")
            .and(toss(with_authorization(true, session.clone())))
            .and(internal_filter(db_config.clone(), session.clone()));
        let check = warp::path("check")
            .and(toss(with_authorization(false, session.clone())))
            .and(check_filter(db_config.clone(), session.clone()));
        warp::any().and(check.or(internal.or(login)))
    }

    pub fn check_filter(
        db_config: Arc<DatabaseConfig>,
        session: Arc<Session>,
    ) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        warp::any()
            .and(warp::get())
            .and(with_db_config(db_config))
            .and(with_session(session))
            .and(warp::body::json())
            .and(end())
            .and_then(handlers::check)
    }

    pub fn login_filter(
        db_config: Arc<DatabaseConfig>,
        session: Arc<Session>,
    ) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        warp::any()
            .and(with_db_config(db_config))
            .and(with_session(session))
            .and(warp::post())
            .and(warp::body::json())
            .and(end())
            .and_then(handlers::login)
    }
}

pub mod handlers {
    use super::*;
    use crate::utils::errors::AuthenticationError::*;

    #[derive(Serialize, Deserialize)]
    pub struct UserSubmission {
        pub name: String,
        pub email: String,
        pub password: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct LoginSubmission {
        pub email: String,
        pub password: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct CheckRequest {
        pub user_id: i64,
        pub permission_id: i64,
    }

    pub async fn check(
        db_config: Arc<DatabaseConfig>,
        session: Arc<Session>,
        check: CheckRequest,
    ) -> Result<impl Reply, Rejection> {
        let connection = get_connection(session)?;
        // TODO:
        // 1. Find permissions the internal user the user belongs to has
        // 2. Find the parent permissions of the requested permission
        // 3. Check if any of the user's role has any of those permissions
        Ok(warp::reply::json(&0))
    }

    pub async fn login(
        db_config: Arc<DatabaseConfig>,
        session: Arc<Session>,
        login: LoginSubmission,
    ) -> Result<impl Reply, Rejection> {
        let connection = get_connection(session)?;
        let user = InternalUser::find_by_email(login.email, &connection)
            .or(Err(warp::reject::custom(InvalidEmail)))?;
        let submitted_hashed = hash_password(login.password, user.salt, db_config.iterations);
        let existing_hashed = user.password;
        ring::constant_time::verify_slices_are_equal(&submitted_hashed, existing_hashed.as_slice())
            .or(Err(warp::reject::custom(InvalidPassword)))?;
        let iuser = InternalUser::update_auth_token(user.id, db_config, &connection)
            .or(Err(warp::reject::custom(CouldNotGenerateAuthToken)))?;
        Ok(warp::reply::json(&json!({
                "authorization_token": iuser.auth_token
        })))
    }
}

#[cfg(test)]
mod tests {}
