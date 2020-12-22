use crate::utils::common::with_predicate;
use warp::{reject, Filter, Rejection};

use crate::{
    database::get_connection,
    database::models::internal_user::InternalUser,
    utils::common::{with_session, Session},
    utils::errors::*,
};

pub fn with_authorization(
    session: Session,
) -> impl Filter<Extract = (InternalUser,), Error = Rejection> + Clone {
    warp::any()
        .and(with_session(session))
        .and(warp::header::optional::<String>("authorization"))
        .and(with_predicate(default))
        .and_then(check_authorized)
}

pub fn with_authorization_no_ret(
    session: Session,
) -> impl Filter<Extract = (), Error = Rejection> + Clone {
    with_authorization(session).map(|_| ()).untuple_one()
}

pub fn with_authorization_admin(
    session: Session,
) -> impl Filter<Extract = (InternalUser,), Error = Rejection> + Clone {
    warp::any()
        .and(with_session(session))
        .and(warp::header::optional::<String>("authorization"))
        .and(with_predicate(admin))
        .and_then(check_authorized)
}

pub fn with_authorization_admin_no_ret(
    session: Session,
) -> impl Filter<Extract = (), Error = Rejection> + Clone {
    with_authorization_admin(session).map(|_| ()).untuple_one()
}

pub fn admin(iuser: InternalUser) -> Result<InternalUser, Rejection> {
    match iuser.admin {
        true => Ok(iuser),
        false => Err(reject::custom(AuthorizationError::Unauthorized)),
    }
}

pub fn default(iuser: InternalUser) -> Result<InternalUser, Rejection> {
    Ok(iuser)
}

pub async fn check_authorized(
    session: Session,
    bearer_token: Option<String>,
    predicate: fn(InternalUser) -> Result<InternalUser, Rejection>,
) -> Result<InternalUser, Rejection> {
    match bearer_token {
        Some(token) => {
            let connection = get_connection(&session)?;
            match InternalUser::find_by_auth_token(token, &connection) {
                Ok(iuser) => predicate(iuser),
                Err(_) => Err(reject::custom(AuthorizationError::InvalidToken)),
            }
        }
        None => Err(reject::custom(AuthorizationError::NoToken)),
    }
}
