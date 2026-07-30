//! 认证模块 SeaORM 实体

pub mod accounts {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "accounts")]
    pub struct Model {
        #[sea_orm(primary_key, column = "id")]
        pub id: Uuid,
        #[sea_orm(column = "name", unique, max_len = 64)]
        pub name: String,
        #[sea_orm(column = "nick", max_len = 64)]
        pub nick: String,
        #[sea_orm(column = "language", max_len = 16)]
        pub language: String,
        #[sea_orm(column = "region", max_len = 16)]
        pub region: String,
        #[sea_orm(column = "is_superuser")]
        pub is_superuser: bool,
        #[sea_orm(column = "activated_at")]
        pub activated_at: Option<DateTime<Utc>>,
        #[sea_orm(column = "created_at")]
        pub created_at: DateTime<Utc>,
        #[sea_orm(column = "updated_at")]
        pub updated_at: DateTime<Utc>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(has_many = "super::account_secrets::Entity")]
        pub secrets,
        #[sea_orm(has_many = "super::account_auth_factor::Entity")]
        pub auth_factors,
        #[sea_orm(has_many = "super::auth_session::Entity")]
        pub sessions,
    }

    impl Related<super::account_secrets::Entity> for Entity {
        fn to() -> RelationDef {
            super::account_secrets::Relation::Account.def()
        }
    }

    impl Related<super::account_auth_factor::Entity> for Entity {
        fn to() -> RelationDef {
            super::account_auth_factor::Relation::Account.def()
        }
    }

    impl Related<super::auth_session::Entity> for Entity {
        fn to() -> RelationDef {
            super::auth_session::Relation::Account.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod account_secrets {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "account_secrets")]
    pub struct Model {
        #[sea_orm(primary_key, column = "id")]
        pub id: Uuid,
        #[sea_orm(column = "account_id")]
        pub account_id: Uuid,
        #[sea_orm(column = "secret_type", max_len = 32)]
        pub secret_type: String,
        #[sea_orm(column = "secret_value", max_len = 256)]
        pub secret_value: String,
        #[sea_orm(column = "enabled_at")]
        pub enabled_at: Option<DateTime<Utc>>,
        #[sea_orm(column = "created_at")]
       
    pub created_at: DateTime<Utc>, }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(back_into = "super::accounts::Entity")]
        pub account

,
    }    impl ActiveModelBehavior for ActiveModel {}
}

pub mod account_auth_factor {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "account_auth_factor")]
    pub struct Model {
        #[sea_orm(primary_key, column = "id")]
        pub id: Uuid,
        #[sea_orm(column = "account_id")]
        pub account_id: Uuid,
        #[sea_orm(column = "factor_type", max_len = 32)]
        pub factor_type: String,
        #[sea_orm(column = "factor_data", max_len = 512)]
        pub factor_data: String,
        #[sea_orm(column = "enabled_at")]
        pub enabled_at: Option<DateTime<Utc>>,
        #[sea_orm(column = "trustworthy")]
        pub trustworthy: i32,
        #[sea_orm(column = "created_at")]
        pub created_at: DateTime<Utc>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(back_into = "super::accounts::Entity")]
        pub account,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod auth_session {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "auth_session")]
    pub struct Model {
        #[sea_orm(primary_key, column = "id")]
        pub id: Uuid,
        #[sea_orm(column = "account_id")]
        pub account_id: Uuid,
        #[sea_orm(column = "device_id", max_len = 128)]
        pub device_id: String,
        #[sea_orm(column = "device_name", max_len = 128)]
        pub device_name: Option<String>,
        #[sea_orm(column = "platform", max_len = 32)]
        pub platform: String,
        #[sea_orm(column = "access_token", max_len = 512)]
        pub access_token: String,
        #[sea_orm(column = "access_expires_at")]
        pub access_expires_at: DateTime<Utc>,
        #[sea_orm(column = "refresh_token", max_len = 512)]
        pub refresh_token: String,
        #[sea_orm(column = "refresh_expires_at")]
        pub refresh_expires_at: DateTime<Utc>,
        #[sea_orm(column = "created_at")]
        pub created_at: DateTime<Utc>,
        #[sea_orm(column = "expired_at")]
        pub expired_at: DateTime<Utc>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(back_into = "super::accounts::Entity")]
        pub account,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod auth_challenge {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "auth_challenge")]
    pub struct Model {
        #[sea_orm(primary_key, column = "id")]
        pub id: Uuid,
        #[sea_orm(column = "account_id")]
        pub account_id: Uuid,
        #[sea_orm(column = "device_id", max_len = 128)]
        pub device_id: String,
        #[sea_orm(column = "device_name", max_len = 128)]
        pub device_name: Option<String>,
        #[sea_orm(column = "platform", max_len = 32)]
        pub platform: String,
        #[sea_orm(column = "ip_address", max_len = 64)]
        pub ip_address: Option<String>,
        #[sea_orm(column = "user_agent", max_len = 512)]
        pub user_agent: Option<String>,
        #[sea_orm(column = "step_total")]
        pub step_total: i32,
        #[sea_orm(column = "step_remain")]
        pub step_remain: i32,
        #[sea_orm(column = "failed_attempts")]
        pub failed_attempts: i32,
        #[sea_orm(column = "expired_at")]
        pub expired_at: Option<DateTime<Utc>>,
        #[sea_orm(column = "approved_at")]
        pub approved_at: Option<DateTime<Utc>>,
        #[sea_orm(column = "declined_at")]
        pub declined_at: Option<DateTime<Utc>>,
        #[sea_orm(column = "created_at")]
        pub created_at: DateTime<Utc>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}