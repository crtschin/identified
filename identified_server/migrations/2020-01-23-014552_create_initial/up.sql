create table internal_user (
    id bigserial not null primary key,
    name text not null,
    email text not null unique,
    password bytea not null,
    salt text not null,
    created_on timestamptz not null,
    last_login timestamptz,
    auth_token text null,
    expires_on timestamptz,
    admin boolean not null default false
);

create table "user" (
  "id" bigserial primary key,
  "name" text,
  "owner_id" bigint not null
);

create table "user_role" (
  "id" bigserial primary key,
  "user_id" bigint not null,
  "role_id" bigint not null
);

create table "role" (
  "id" bigserial primary key,
  "name" ltree not null,
  "owner_id" bigint
);

create table "role_permission" (
  "id" bigserial primary key,
  "role_id" bigint not null,
  "permission_id" bigint not null
);

create table "permission" (
  "id" bigserial primary key,
  "name" ltree not null,
  "owner_id" bigint not null
);

create table "user_permission" (
  "id" bigserial primary key,
  "user_id" bigint not null,
  "permission_id" bigint not null
);

alter table "user" add constraint "user_fk_owner_id" foreign key ("owner_id") references "internal_user" ("id");
alter table "user_role" add constraint "user_role_fk_user_id" foreign key ("user_id") references "user" ("id");
alter table "user_role" add constraint "user_role_fk_role_id" foreign key ("role_id") references "role" ("id");
alter table "role" add constraint "role_fk_owner_id" foreign key ("owner_id") references "internal_user" ("id");
alter table "role_permission" add constraint "role_permission_fk_role_id" foreign key ("role_id") references "role" ("id");
alter table "role_permission" add constraint "role_permission_fk_permission_id" foreign key ("permission_id") references "permission" ("id");
alter table "permission" add constraint "permission_fk_owner_id" foreign key ("owner_id") references "internal_user" ("id");
alter table "user_permission" add constraint "user_permission_fk_user_id" foreign key ("user_id") references "user" ("id");
alter table "user_permission" add constraint "user_permission_fk_permission_id" foreign key ("permission_id") references "permission" ("id");
