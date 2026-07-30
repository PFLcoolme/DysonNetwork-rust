//!  数据库连接与 Schema定义

pub mod schema {
    diesel::table! {
        actors (id) {
            id -> Uuid,
            name -> Text,
            display_name -> Nullable<Text>,
            summary -> Nullable<Text>,
            public_key_pem -> Text,
            private_key_pem -> Text,
            inbox_url -> Text,
            outbox_url -> Text,
            shared_inbox_url -> Nullable<Text>,
            endpoint_url -> Text,
            actor_type -> Text,
            is_instance -> Bool,
            avatar_url -> Nullable<Text>,
            header_image_url -> Nullable<Text>,
            deleted -> Bool,
            created_at -> Timestamp,
            updated_at -> Timestamp,
        }
    }

    diesel::table! {
        activities (id) {
            id -> Uuid,
            actor_id -> Uuid,
            activity_type -> Text,
            raw_json -> Text,
            object_type -> Nullable<Text>,
            object_id -> Nullable<Text>,
            to_vec -> Array<Nullable<Text>>,
            cc_vec -> Array<Nullable<Text>>,
            in_reply_to -> Nullable<Uuid>,
            deleted -> Bool,
            created_at -> Timestamp,
        }
    }

    diesel::table! {
        posts (id) {
            id -> Uuid,
            actor_id -> Uuid,
            title -> Text,
            content -> Nullable<Text>,
            content_type -> Text,
            visibility -> Text,
            in_reply_to -> Nullable<Uuid>,
            repost_of -> Nullable<Uuid>,
            url -> Text,
            ap_id -> Text,
            deleted -> Bool,
            created_at -> Timestamp,
            updated_at -> Timestamp,
        }
    }

    diesel::joinable!(activities -> actors (actor_id));
    diesel::joinable!(posts -> actors (actor_id));

    diesel::allow_tables_to_appear_in_same_query!(actors, activities, posts);
}

use diesel::prelude::*;
use diesel::r2d2::{ConnectionPool, Pool, Builder};
use diesel::MysqlConnection;

pub type MysqlPool = Pool<MysqlConnection>;

pub fn create_pool(database_url: &str) -> MysqlPool {
    let pool = Pool::builder::<MysqlConnection>()
        .max_size(10)
        .connection_customizer(Box::new(ConnectionConfig {
            prepare
       _connections: 1, }))
        .build(database_url)
        .expect("Failed to create pool");
    
    pool
}

struct ConnectionConfig {
    prepare_connections: u32,
}

impl diesel::r2d2::CustomizeConnection<MysqlConnection, diesel::r2d2::Error> for ConnectionConfig {
    fn on_acquire(&self, _conn: &mut MysqlConnection, _id: &str) -> Result<(), diesel::r2d2::Error> {
        Ok(())
    }
}

lazy_static::lazy_static! {
    pub static ref POOL: MysqlPool = {
        create_pool(
            &std::env::var("DATABASE_URL").unwrap_or_else(|_| "mysql://root:@127.0.0.1:3306/solsynth".to_string())
        )
    };
}

pub fn get_pool() -> MysqlPool {
    (*POOL).clone()
}