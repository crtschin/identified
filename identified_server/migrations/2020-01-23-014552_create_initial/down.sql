-- this file should undo anything in `up.sql`
alter table "user" drop constraint if exists "user_fk_owner_id";
alter table "user_role" drop constraint if exists "user_role_fk_user_id";
alter table "user_role" drop constraint if exists "user_role_fk_role_id";
alter table "role" drop constraint if exists "role_fk_parent_id";
alter table "role" drop constraint if exists "role_fk_owner_id";
alter table "role_permission" drop constraint if exists "role_permission_fk_role_id";
alter table "role_permission" drop constraint if exists "role_permission_fk_permission_id";
alter table "permission" drop constraint if exists "permission_fk_parent_id";
alter table "permission" drop constraint if exists "permission_fk_owner_id";
alter table "user_permission" drop constraint if exists "user_permission_fk_user_id";
alter table "user_permission" drop constraint if exists "user_permission_fk_permission_id";

drop table if exists "internal_user";
drop table if exists "user";
drop table if exists "user_role";
drop table if exists "role";
drop table if exists "role_permission";
drop table if exists "permission";
drop table if exists "user_permission";