#![deny(warnings)]

use diesel_ltree::Ltree;
pub mod internal_user;

#[derive(Queryable)]
pub struct Namespace {
    pub id: i64,
    pub name: String,
    pub iuserid: i64,
}

#[derive(Queryable)]
pub struct User {
    pub id: i64,
    pub name: String,
}

#[derive(Queryable)]
pub struct UserRoles {
    pub id: i64,
    pub user_id: i64,
    pub role_id: Role,
}

#[derive(Queryable)]
pub struct Role {
    pub id: i64,
    pub name: String,
}

pub struct RolePermission {
    pub id: i64,
    pub role_id: i64,
    pub permission_id: Role,
}

#[derive(Queryable)]
pub struct Permission {
    pub id: i64,
    pub name: String,
    pub parent_id: i64,
    pub permission: Ltree,
}
