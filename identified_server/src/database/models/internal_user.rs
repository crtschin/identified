use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
pub struct NewInternalUser {
    pub name: String,
    pub email: String,
    pub password: Vec<u8>,
    pub salt: String,
    pub created_on: DateTime<Utc>,
    pub admin: bool,
}

#[derive(AsChangeset, Identifiable)]
#[table_name = "internal_user"]
pub struct ChangeLogin {
    pub id: i64,
    pub email: String,
    pub password: Vec<u8>,
    pub salt: String,
}

#[derive(AsChangeset, Identifiable)]
#[table_name = "internal_user"]
pub struct UpdateToken {
    pub id: i64,
    pub auth_token: String,
    pub expires_on: DateTime<Utc>,
}

#[derive(AsChangeset, Identifiable)]
#[table_name = "internal_user"]
pub struct SuccessfullLogin {
    pub id: i64,
    pub last_login: DateTime<Utc>,
    pub auth_token: String,
    pub expires_on: DateTime<Utc>,
}

// TODO: Swap Error with a custom error type
impl InternalUser {
    // pub fn create_root_user(
    //     root_password: String,
    //     root_salt: String,
    //     iterations: NonZeroU32,
    //     connection: &PgConnection,
    // ) {
    //     let hashed = hash_password(root_password, root_salt.clone(), iterations);
    //     // TODO: load this from a config file
    //     let root_user = NewInternalUser {
    //         name: "root".to_string(),
    //         email: "root@admin.com".to_string(),
    //         password: hashed.to_vec(),
    //         salt: root_salt,
    //         created_on: Utc::now(),
    //     };

    //     diesel::insert_into(internal_user::table)
    //         .values(vec![root_user])
    //         .get_result::<InternalUser>(connection)
    //         .expect("Error registering new user");
    // }

    pub fn create_with_values(
        user_name: String,
        user_email: String,
        user_password: String,
        is_admin: bool,
        db_config: Arc<DatabaseConfig>,
        connection: &PgConnection,
    ) -> Result<InternalUser, Error> {
        // First check if the user already exists
        let possible_user = InternalUser::find_by_email(user_email.clone(), connection);
        match possible_user {
            Ok(_) => {
                return Err(Error::Input(InputError {
                    fields: (String::from("email"), ValidationError::AlreadyExists),
                }));
            }
            Err(err) => match err {
                diesel::result::Error::NotFound => {
                    let user_salt = random_string(db_config.salt_length);
                    let hashed =
                        hash_password(user_password, user_salt.clone(), db_config.iterations);
                    let new_user = NewInternalUser {
                        name: user_name,
                        email: user_email,
                        password: hashed.to_vec(),
                        salt: user_salt,
                        created_on: Utc::now(),
                        admin: is_admin,
                    };
                    InternalUser::create(new_user, connection).map_err(|e| {
                        let msg = format!("{}", e);
                        Error::Db(DatabaseConnectionError { msg })
                    })
                }
                _ => {
                    let msg = format!("{}", err);
                    Err(Error::Db(DatabaseConnectionError { msg }))
                }
            },
        }
    }

    pub fn create(
        new: NewInternalUser,
        connection: &PgConnection,
    ) -> Result<InternalUser, diesel::result::Error> {
        diesel::insert_into(internal_user::table)
            .values(vec![new])
            .get_result::<InternalUser>(connection)
    }

    pub fn all(connection: &PgConnection) -> Vec<InternalUser> {
        dsl::internal_user
            .load(connection)
            .expect("Error loading users")
    }

    pub fn find(
        iuser_id: i64,
        connection: &PgConnection,
    ) -> Result<InternalUser, diesel::result::Error> {
        dsl::internal_user.find(iuser_id).first(connection)
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

    pub fn update_login(
        by_id: i64,
        new_email: String,
        new_password: String,
        db_config: Arc<DatabaseConfig>,
        connection: &PgConnection,
    ) -> Result<InternalUser, diesel::result::Error> {
        let user_salt = random_string(db_config.salt_length);
        let hashed = hash_password(new_password, user_salt.clone(), db_config.iterations);
        diesel::update(internal_user::table)
            .set(ChangeLogin {
                id: by_id,
                email: new_email,
                password: hashed.to_vec(),
                salt: user_salt,
            })
            .get_result(connection)
    }

    pub fn successfull_login(
        login: SuccessfullLogin,
        connection: &PgConnection,
    ) -> Result<InternalUser, diesel::result::Error> {
        diesel::update(internal_user::table)
            .set(login)
            .get_result(connection)
    }
}
