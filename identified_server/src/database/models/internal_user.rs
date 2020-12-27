use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use warp::*;

use crate::database::models::*;
use crate::database::schema::internal_user::*;
use crate::database::DatabaseConfig;
use crate::utils::common::{hash_password, random_string};
use crate::utils::errors::*;

#[derive(Queryable, Serialize, Deserialize)]
pub struct InternalUser {
    pub id: i64,
    pub name: String,
    pub email: String,
    #[serde(with = "serde_bytes")]
    pub password: Vec<u8>,
    pub salt: String,
    pub created_on: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub auth_token: Option<String>,
    pub expires_on: Option<DateTime<Utc>>,
    pub admin: bool,
}

#[derive(Insertable)]
#[table_name = "internal_user"]
pub struct CreateInternalUser {
    pub name: String,
    pub email: String,
    pub password: Vec<u8>,
    pub salt: String,
    pub created_on: DateTime<Utc>,
    pub admin: bool,
}

#[derive(AsChangeset, Identifiable)]
#[table_name = "internal_user"]
pub struct UpdateInternalUser {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub password: Vec<u8>,
    pub salt: String,
}

#[derive(Serialize, Deserialize)]
pub struct SubmitInternalUser {
    pub name: String,
    pub email: String,
    pub password: String,
}

// TODO: Swap Error with a custom error type
impl InternalUser {
    pub fn all(connection: &PgConnection) -> Result<Vec<InternalUser>, diesel::result::Error> {
        dsl::internal_user.load(connection)
    }

    pub fn create(
        new: SubmitInternalUser,
        is_admin: bool,
        db_config: Arc<DatabaseConfig>,
        connection: &PgConnection,
    ) -> Result<InternalUser, diesel::result::Error> {
        let user_salt = random_string(db_config.salt_length);
        let hashed = hash_password(new.password, user_salt.clone(), db_config.iterations);
        diesel::insert_into(internal_user::table)
            .values(vec![CreateInternalUser {
                name: new.name,
                email: new.email,
                password: hashed.to_vec(),
                salt: user_salt,
                created_on: Utc::now(),
                admin: is_admin,
            }])
            .get_result::<InternalUser>(connection)
    }

    pub fn find_by_id(
        by_id: i64,
        connection: &PgConnection,
    ) -> Result<InternalUser, diesel::result::Error> {
        dsl::internal_user.find(by_id).first(connection)
    }

    pub fn find_by_email(
        by_email: String,
        connection: &PgConnection,
    ) -> Result<InternalUser, diesel::result::Error> {
        dsl::internal_user
            .filter(email.eq(by_email))
            .first(connection)
    }

    pub fn find_by_auth_token(
        by_auth_token: String,
        connection: &PgConnection,
    ) -> Result<InternalUser, diesel::result::Error> {
        dsl::internal_user
            .filter(auth_token.eq(by_auth_token))
            .first(connection)
    }

    pub fn delete(by_id: i64, connection: &PgConnection) -> Result<usize, diesel::result::Error> {
        diesel::delete(dsl::internal_user.filter(id.eq(by_id))).execute(connection)
    }

    pub fn update(
        by_id: i64,
        new: SubmitInternalUser,
        db_config: Arc<DatabaseConfig>,
        connection: &PgConnection,
    ) -> Result<InternalUser, diesel::result::Error> {
        let user_salt = random_string(db_config.salt_length);
        let hashed = hash_password(new.password, user_salt.clone(), db_config.iterations);
        let new_token = random_string(db_config.api_key_length);
        diesel::update(internal_user::table)
            .set((
                UpdateInternalUser {
                    id: by_id,
                    name: new.name,
                    email: new.email,
                    password: hashed.to_vec(),
                    salt: user_salt,
                },
                auth_token.eq(new_token),
            ))
            .get_result(connection)
    }

    pub fn update_auth_token(
        by_id: i64,
        db_config: Arc<DatabaseConfig>,
        connection: &PgConnection,
    ) -> Result<InternalUser, diesel::result::Error> {
        let new_token = random_string(db_config.api_key_length);
        diesel::update(internal_user::table.filter(id.eq(by_id)))
            .set((
                auth_token.eq(new_token),
                last_login.eq(Utc::now()),
                expires_on.eq(Utc::now() + Duration::minutes(120)),
            ))
            .get_result(connection)
    }
}
