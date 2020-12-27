pub mod models;
pub mod schema;
pub mod seed;

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PoolError, PooledConnection};
use dotenv::dotenv;
use ring::rand::SystemRandom;
use std::sync::Arc;
use std::{env, num};
use warp::{reject, Rejection};

use crate::utils::common::Session;
use crate::utils::errors::DbError::DatabaseConnectionError;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

// TODO: Make it harder to shoot yourself in the foot by using different number
// of hashing iterations when passwords have already been defined
pub struct DatabaseConfig {
    pub iterations: num::NonZeroU32,
    pub rng: SystemRandom,
    pub salt_length: usize,
    pub api_key_length: usize,
}

impl DatabaseConfig {
    pub fn default() -> DatabaseConfig {
        DatabaseConfig {
            iterations: num::NonZeroU32::new(100).unwrap(),
            rng: SystemRandom::new(),
            salt_length: 12,
            api_key_length: 12,
        }
    }
}

pub fn get_connection(session: Arc<Session>) -> Result<PgPooledConnection, Rejection> {
    match session.connection_pool.get() {
        Ok(connection) => Ok(connection),
        Err(err) => {
            return Err(reject::custom(DatabaseConnectionError(format!("{}", err))));
        }
    }
}

fn init_pool(database_url: &str) -> Result<PgPool, PoolError> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder().build(manager)
}

pub fn establish_connection() -> PgPool {
    // Load .env into the environment variables
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    init_pool(&database_url).expect("Error connecting to {}")
}
