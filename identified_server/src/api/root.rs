use futures::{FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc;
use warp::{path::end, Filter, Rejection, Reply};

use crate::{
    api::helpers::authorization::*,
    api::internal::filters::main_filter as internal_filter,
    database::models::internal_user::InternalUser,
    database::{get_connection, DatabaseConfig},
    utils::common::*,
};

static CONNECTION_ID: AtomicUsize = AtomicUsize::new(1);

type PermissionHashMap<T> = Arc<Mutex<HashMap<i64, HashMap<usize, T>>>>;
type PermissionStreams =
    PermissionHashMap<mpsc::UnboundedSender<Result<PermissionUpdate, warp::Error>>>;

#[derive(Serialize)]
pub struct PermissionUpdate {
    pub user_id: i64,
    pub permission_id: i64,
    pub allowed: bool,
}

pub mod filters {
    use super::*;

    pub fn main_filter(
        db_config: Arc<DatabaseConfig>,
        session: Arc<Session>,
        permission_streams: PermissionStreams,
    ) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        let login = warp::path("login").and(login_filter(
            db_config.clone(),
            session.clone(),
            permission_streams.clone(),
        ));
        let internal = warp::path("internal")
            .and(toss(with_authorization(true, session.clone())))
            .and(internal_filter(db_config.clone(), session.clone()));
        let check = warp::path("check")
            .and(toss(with_authorization(false, session.clone())))
            .and(check_filter(
                db_config.clone(),
                session.clone(),
                permission_streams.clone(),
            ));
        let subscribe = warp::path("subscribe")
            .and(warp::ws())
            .and(with_authorization(false, session.clone()))
            .and(warp::body::json())
            .and(warp::any().map(move || permission_streams.clone()))
            .map(
                |ws: warp::ws::Ws, iuser, requested_permissions, permission_streams| {
                    ws.on_upgrade(move |socket| {
                        handlers::new_subscription(
                            socket,
                            requested_permissions,
                            iuser,
                            permission_streams,
                        )
                    })
                },
            );
        warp::any().and(check.or(internal.or(login.or(subscribe))))
    }

    fn check_filter(
        db_config: Arc<DatabaseConfig>,
        session: Arc<Session>,
        permission_streams: PermissionStreams,
    ) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        warp::any()
            .and(warp::get())
            .and(with_db_config(db_config))
            .and(with_session(session))
            .and(warp::body::json())
            .and(end())
            .and_then(handlers::check)
    }

    fn login_filter(
        db_config: Arc<DatabaseConfig>,
        session: Arc<Session>,
        permission_streams: PermissionStreams,
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

mod handlers {
    use super::*;
    use crate::utils::errors::AuthenticationError::*;
    use crate::utils::errors::ServerError::SerializationError;
    use warp::{filters::ws::Message, ws::WebSocket};

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

    pub async fn new_subscription(
        ws: WebSocket,
        permission_ids: Vec<i64>,
        iuser: InternalUser,
        permission_streams: PermissionStreams,
    ) {
        // Create a new unbounded channel where we'll send the messages with
        // permission updates
        let (sender, receiver) = mpsc::unbounded_channel::<Result<PermissionUpdate, warp::Error>>();

        // Split the socket into a sender and receive of messages.
        let (ws_sink, mut ws_stream) = ws.split();

        tokio::task::spawn(
            receiver
                .map(|update| {
                    update.map(|ok: PermissionUpdate| {
                        Message::text(serde_json::to_string(&ok).unwrap())
                    })
                })
                .forward(ws_sink)
                .map(|result| {
                    if let Err(e) = result {
                        eprintln!("Websocket sent error: {}", e);
                    }
                }),
        );

        // we want to keep track of where we insert the connections so we can
        // remove them later
        let mut to_disconnect = HashMap::<i64, usize>::default();

        // register the socket to the requested permission streams
        for permission_id in permission_ids {
            let new_id = CONNECTION_ID.fetch_add(1, Ordering::Relaxed);
            to_disconnect.insert(permission_id, new_id);
            let mut permission_streams = permission_streams.lock().unwrap();
            let streams_for_permission = permission_streams
                .entry(permission_id)
                .or_insert(HashMap::new());
            streams_for_permission.insert(new_id, sender.clone());
        }

        while let Some(result) = ws_stream.next().await {
            match result {
                Ok(msg) => {}
                Err(e) => {
                    eprintln!("Websocket error (iuser_id = {}): {}", iuser.id, e);
                    break;
                }
            }
        }

        // remove the disconnected socket from our maps
        for (k, v) in to_disconnect {
            let mut permission_streams = permission_streams.lock().unwrap();
            permission_streams.get_mut(&k).and_then(|m| m.remove(&v));
        }
    }
}

#[cfg(test)]
mod tests {}
