use identified::{
    api::internal::filters::internal_filter,
    database::models::internal_user::InternalUser,
    database::{establish_connection, get_connection, DatabaseConfig},
    utils::common::Session,
};
use listenfd::ListenFd;
use ring::rand::*;
use std::num::NonZeroU32;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Collect command line arguments
    // let args: Vec<String> = std::env::args().collect();
    // let is_debug = !args.iter().any(|arg| arg.eq("--release"));

    // Start the server and add routes
    // TODO: Insert root user if they don't already exist
    let db_pool = establish_connection();
    let db_config = Arc::new(DatabaseConfig {
        iterations: NonZeroU32::new(1000).unwrap(),
        rng: SystemRandom::new(),
        salt_length: 24,
        api_key_length: 30,
    });

    let session = Session {
        connection_pool: db_pool,
    };

    // Either create a new root user or update the password of the root user
    // TODO: The email and password fields of the user should be externally
    //          configurable
    //
    // Should cause internal error if a connection could not be acquired.
    let connection = get_connection(&session).unwrap();
    let possible = InternalUser::find_by_email(String::from("root@admin.com"), &connection);
    let _user = match possible {
        // Update existing
        Ok(u) => InternalUser::update_login(
            u.id,
            "root@admin.com".to_string(),
            "password".to_string(),
            db_config.clone(),
            &connection,
        )
        .expect("Failure to update root user's password"),
        Err(_) => InternalUser::create_with_values(
            "root".to_string(),
            "root@admin.com".to_string(),
            "password".to_string(),
            true,
            db_config.clone(),
            &connection,
        )
        .expect("Failure to create root user"),
    };

    // If the program was not built using release, try and use listenfd for
    // hot-reloading
    let server = warp::serve(internal_filter(db_config, session));
    if let Ok(profile) = std::env::var("PROFILE") {
        if let "release" = profile.as_str() {
            server.run(([127, 0, 0, 1], 3000)).await;
            return;
        }
    }

    let mut listenfd = ListenFd::from_env();
    if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        let mut listener = tokio::net::TcpListener::from_std(l).unwrap();
        server.run_incoming(listener.incoming()).await;
    } else {
        server.run(([127, 0, 0, 1], 3000)).await;
    };
}
