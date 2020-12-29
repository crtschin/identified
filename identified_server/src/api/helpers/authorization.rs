use crate::utils::common::with_predicate;
use std::sync::Arc;
use warp::{reject, Filter, Rejection};

use crate::{
    database::get_connection,
    database::models::internal_user::InternalUser,
    utils::common::{with, Session},
    utils::errors::*,
};

pub fn admin(iuser: InternalUser) -> Result<InternalUser, Rejection> {
    match iuser.admin {
        true => Ok(iuser),
        false => Err(reject::custom(AuthorizationError::Unauthorized)),
    }
}

pub fn default(iuser: InternalUser) -> Result<InternalUser, Rejection> {
    Ok(iuser)
}

pub fn with_authorization(
    requires_admin: bool,
    session: Arc<Session>,
) -> impl Filter<Extract = (InternalUser,), Error = Rejection> + Clone {
    let predicate = match requires_admin {
        true => admin,
        false => default,
    };
    warp::any()
        .and(with(session))
        .and(warp::header::optional::<String>("authorization"))
        .and(with_predicate(predicate))
        .and_then(check_authorized)
}

pub async fn check_authorized(
    session: Arc<Session>,
    bearer_token: Option<String>,
    predicate: fn(InternalUser) -> Result<InternalUser, Rejection>,
) -> Result<InternalUser, Rejection> {
    match bearer_token {
        Some(token) => {
            let connection = get_connection(session)?;
            match InternalUser::find_by_auth_token(token, &connection) {
                Ok(iuser) => predicate(iuser),
                Err(_) => Err(reject::custom(AuthorizationError::InvalidToken)),
            }
        }
        None => Err(reject::custom(AuthorizationError::NoToken)),
    }
}
