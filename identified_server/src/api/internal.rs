use std::sync::Arc;
use warp::{path::end, Filter, Rejection, Reply};

use crate::{
    database::models::internal_user::InternalUser,
    database::{get_connection, DatabaseConfig},
    utils::common::*,
    utils::errors::*,
};

pub mod filters {
    use super::*;

    pub fn main_filter(
        db_config: Arc<DatabaseConfig>,
        session: Arc<Session>,
    ) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        warp::any().and(
            all_filter(session.clone())
                .or(create_filter(db_config.clone(), session.clone()))
                .or(update_filter(db_config.clone(), session.clone()))
                .or(delete_filter(session.clone())),
        )
    }

    pub fn all_filter(
        session: Arc<Session>,
    ) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        warp::any()
            .and(warp::get())
            .and(with_session(session))
            .and(end())
            .and_then(handlers::all)
    }

    pub fn create_filter(
        db_config: Arc<DatabaseConfig>,
        session: Arc<Session>,
    ) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        warp::any()
            .and(warp::post())
            .and(warp::body::content_length_limit(1024 * 16))
            .and(warp::body::json())
            .and(with_db_config(db_config))
            .and(with_session(session))
            .and(end())
            .and_then(handlers::create)
    }

    pub fn update_filter(
        db_config: Arc<DatabaseConfig>,
        session: Arc<Session>,
    ) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        warp::any()
            .and(warp::patch())
            .and(warp::body::content_length_limit(1024 * 16))
            .and(warp::body::json())
            .and(with_db_config(db_config))
            .and(with_session(session))
            .and(end())
            .and_then(handlers::update)
    }

    pub fn delete_filter(
        session: Arc<Session>,
    ) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        warp::any()
            .and(warp::delete())
            .and(with_session(session))
            .and(end())
            .and_then(handlers::all)
    }
}

pub mod handlers {
    use super::*;
    use crate::database::models::internal_user::SubmitInternalUser;
    use crate::utils::common::WithId;
    use crate::utils::errors::DbError::*;
    use http;

    pub async fn all(session: Arc<Session>) -> Result<impl Reply, Rejection> {
        let connection = get_connection(session)?;
        let results = InternalUser::all(&connection)
            .map_err(|e| warp::reject::custom(DbError::DatabaseQueryError(format!("{}", e))))?;
        Ok(warp::reply::json(&results))
    }

    pub async fn create(
        submitted: SubmitInternalUser,
        db_config: Arc<DatabaseConfig>,
        session: Arc<Session>,
    ) -> Result<impl Reply, Rejection> {
        let connection = get_connection(session)?;
        InternalUser::create(submitted, false, db_config, &connection)
            .map_err(|e| warp::reject::custom(DatabaseQueryError(format!("{}", e))))?;
        Ok(http::StatusCode::OK)
    }

    pub async fn update(
        submitted: WithId<SubmitInternalUser>,
        db_config: Arc<DatabaseConfig>,
        session: Arc<Session>,
    ) -> Result<impl Reply, Rejection> {
        let connection = get_connection(session)?;
        let results =
            InternalUser::update(submitted.id, submitted.contained, db_config, &connection)
                .map_err(|e| warp::reject::custom(DbError::DatabaseQueryError(format!("{}", e))))?;
        Ok(warp::reply::json(&results))
    }

    pub async fn delete(by_id: i64, session: Arc<Session>) -> Result<impl Reply, Rejection> {
        let connection = get_connection(session)?;
        let results = InternalUser::delete(by_id, &connection)
            .map_err(|e| warp::reject::custom(DbError::DatabaseQueryError(format!("{}", e))))?;
        Ok(warp::reply::json(&results))
    }
}

#[cfg(test)]
mod tests {}
