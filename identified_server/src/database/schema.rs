table! {
    use diesel::sql_types::*;
    use diesel_ltree::*;

    internal_user (id) {
        id -> Int8,
        name -> Text,
        email -> Text,
        password -> Bytea,
        salt -> Text,
        created_on -> Timestamptz,
        last_login -> Nullable<Timestamptz>,
        auth_token -> Nullable<Text>,
        expires_on -> Nullable<Timestamptz>,
        admin -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_ltree::*;

    permission (id) {
        id -> Int8,
        name -> Ltree,
        owner_id -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_ltree::*;

    role (id) {
        id -> Int8,
        name -> Ltree,
        owner_id -> Nullable<Int8>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_ltree::*;

    role_permission (id) {
        id -> Int8,
        role_id -> Int8,
        permission_id -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_ltree::*;

    user (id) {
        id -> Int8,
        name -> Nullable<Text>,
        owner_id -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_ltree::*;

    user_permission (id) {
        id -> Int8,
        user_id -> Int8,
        permission_id -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_ltree::*;

    user_role (id) {
        id -> Int8,
        user_id -> Int8,
        role_id -> Int8,
    }
}

joinable!(permission -> internal_user (owner_id));
joinable!(role -> internal_user (owner_id));
joinable!(role_permission -> permission (permission_id));
joinable!(role_permission -> role (role_id));
joinable!(user -> internal_user (owner_id));
joinable!(user_permission -> permission (permission_id));
joinable!(user_permission -> user (user_id));
joinable!(user_role -> role (role_id));
joinable!(user_role -> user (user_id));

allow_tables_to_appear_in_same_query!(
    internal_user,
    permission,
    role,
    role_permission,
    user,
    user_permission,
    user_role,
);
