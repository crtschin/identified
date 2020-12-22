use chrono::{Duration, Utc};
use diesel::prelude::RunQueryDsl;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use warp::{path::end, reject, Filter, Rejection, Reply};

use crate::{
    api::authorization::with_authorization_no_ret,
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
        let all = warp::path("all").and(all_filter(session.clone()));
        warp::any().and(all)
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
}

pub mod handlers {
    use super::*;
    pub async fn all(session: Session) -> Result<impl Reply, Rejection> {
        let connection = get_connection(&session)?;
        let results = InternalUser::all(&connection);
        Ok(warp::reply::json(&results))
    }
}

#[cfg(test)]
mod tests {}
