use chrono::{Duration, Utc};
use diesel::prelude::RunQueryDsl;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use warp::{path::end, reject, Filter, Rejection, Reply};

use crate::{
    api::helpers::authorization::{with_authorization_admin_no_ret, with_authorization_no_ret},
    api::internal::filters::main_filter as internal_filter,
    database::models::internal_user::{InternalUser, NewInternalUser, SuccessfullLogin},
    database::schema,
    database::{get_connection, DatabaseConfig},
    utils::common::{hash_password, random_string, with_db_config, with_session, Session},
    utils::errors::*,
};

pub mod filters {
    use super::*;

    pub fn main_filter(
        db_config: Arc<DatabaseConfig>,
        session: Session,
    ) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        let login = warp::path("login").and(login_filter(db_config.clone(), session.clone()));
        let register = warp::path("register")
            .and(with_authorization_admin_no_ret(session.clone()))
            .and(register_filter(db_config.clone(), session.clone()));
        let all = warp::path("all")
            .and(with_authorization_admin_no_ret(session.clone()))
            .and(all_filter(session.clone()));
        let check = warp::path("check")
            .and(with_authorization_no_ret(session.clone()))
            .and(check_filter(db_config.clone(), session.clone()));
        let internal = warp::path("internal")
            .and(with_authorization_no_ret(session.clone()))
            .and(internal_filter(db_config, session));
        warp::any().and(check.or(internal.or(login.or(all.or(register)))))
    }

    pub fn all_filter(
        session: Session,
    ) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        warp::any()
            .and(warp::get())
            .and(with_session(session))
            .and(end())
            .and_then(handlers::all)
    }

    pub fn check_filter(
        db_config: Arc<DatabaseConfig>,
        session: Session,
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
        session: Session,
    ) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        warp::any()
            .and(with_db_config(db_config))
            .and(with_session(session))
            .and(warp::post())
            .and(warp::body::json())
            .and(end())
            .and_then(handlers::login)
    }

    pub fn register_filter(
        db_config: Arc<DatabaseConfig>,
        session: Session,
    ) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        warp::any()
            .and(with_db_config(db_config))
            .and(with_session(session))
            .and(warp::post())
            .and(warp::body::json())
            .and(end())
            .and_then(handlers::register)
    }
}

pub mod handlers {
    use super::*;
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

    pub async fn all(session: Session) -> Result<impl Reply, Rejection> {
        let connection = get_connection(&session)?;
        let results = InternalUser::all(&connection);
        Ok(warp::reply::json(&results))
    }

    pub async fn check(
        db_config: Arc<DatabaseConfig>,
        session: Session,
        check: CheckRequest,
    ) -> Result<impl Reply, Rejection> {
        let connection = get_connection(&session)?;
        // TODO:
        // 1. Find permissions the internal user the user belongs to has
        // 2. Find the parent permissions of the requested permission
        // 3. Check if any of the user's role has any of those permissions
        Ok(warp::reply::json(&0))
    }

    pub async fn login(
        db_config: Arc<DatabaseConfig>,
        session: Session,
        login: LoginSubmission,
    ) -> Result<impl Reply, Rejection> {
        let connection = get_connection(&session)?;
        let possible_user = InternalUser::find_by_email(login.email.clone(), &connection);
        match possible_user {
            Ok(user) => {
                let submitted_hashed =
                    hash_password(login.password, user.salt.clone(), db_config.iterations);
                let existing_hashed = user.password.clone();
                match ring::constant_time::verify_slices_are_equal(
                    &submitted_hashed,
                    existing_hashed.as_slice(),
                ) {
                    Ok(_) => {
                        let success = SuccessfullLogin {
                            id: user.id,
                            expires_on: Utc::now() + Duration::minutes(60),
                            last_login: Utc::now(),
                            auth_token: random_string(db_config.api_key_length).to_string(),
                        };
                        // TODO: Error handle (unsuccessfull update auth_token)
                        let auth_token = InternalUser::successfull_login(success, &connection)
                            .unwrap()
                            .auth_token;
                        Ok(warp::reply::json(&json!({ "auth_token": auth_token })))
                    }
                    Err(_) => todo!(),
                }
            }
            Err(_) => todo!(),
        }
    }

    // TODO: should be user not internaluser
    pub async fn register(
        db_config: Arc<DatabaseConfig>,
        session: Session,
        user: UserSubmission,
    ) -> Result<impl Reply, Rejection> {
        let salt = random_string(db_config.salt_length);
        let hashed = hash_password(user.password.clone(), salt.clone(), db_config.iterations);
        let new_user = NewInternalUser {
            name: user.name.clone(),
            email: user.email.clone(),
            password: hashed.to_vec(),
            admin: false,
            salt: salt,
            created_on: chrono::Utc::now(),
        };

        let connection = get_connection(&session)?;
        let possible_user = InternalUser::find_by_email(user.email.clone(), &connection);
        match possible_user {
            Ok(_) => {
                // TODO: Error handling email already exists
                Err(reject::custom(InputError {
                    fields: (String::from("email"), ValidationError::Required),
                }))
            }
            Err(_) => {
                let inserted: InternalUser = diesel::insert_into(schema::internal_user::table)
                    .values(vec![new_user])
                    .get_result(&connection)
                    .expect("Error registering new user");
                Ok(warp::reply::json(&inserted))
            }
        }
    }
}

#[cfg(test)]
mod tests {}
