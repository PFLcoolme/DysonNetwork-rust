//! 认证模块 SeaORM 实体

pub mod accounts {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "accounts")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: Uuid,
        #[sea_orm(column_name = "name", unique)]
        pub name: String,
        #[sea_orm(column_name = "nick")]
        pub nick: String,
        #[sea_orm(column_name = "language")]
        pub language: String,
        #[sea_orm(column_name = "region")]
        pub region: String,
        #[sea_orm(column_name = "is_superuser")]
        pub is_superuser: bool,
        #[sea_orm(column_name = "activated_at")]
        pub activated_at: Option<DateTime>,
        #[sea_orm(column_name = "created_at")]
        pub created_at: DateTime,
        #[sea_orm(column_name = "updated_at")]
        pub updated_at: DateTime,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod account_secrets {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "account_secrets")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: Uuid,
        #[sea_orm(column_name = "account_id")]
        pub account_id: Uuid,
        #[sea_orm(column_name = "secret_type")]
        pub secret_type: String,
        #[sea_orm(column_name = "secret_value")]
        pub secret_value: String,
        #[sea_orm(column_name = "enabled_at")]
        pub enabled_at: Option<DateTime>,
        #[sea_orm(column_name = "created_at")]
        pub created_at: DateTime,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod account_auth_factor {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "account_auth_factor")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: Uuid,
        #[sea_orm(column_name = "account_id")]
        pub account_id: Uuid,
        #[sea_orm(column_name = "factor_type")]
        pub factor_type: String,
        #[sea_orm(column_name = "factor_data")]
        pub factor_data: String,
        #[sea_orm(column_name = "enabled_at")]
        pub enabled_at: Option<DateTime>,
        #[sea_orm(column_name = "trustworthy")]
        pub trustworthy: i32,
        #[sea_orm(column_name = "created_at")]
        pub created_at: DateTime,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod auth_session {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "auth_session")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: Uuid,
        #[sea_orm(column_name = "account_id")]
        pub account_id: Uuid,
        #[sea_orm(column_name = "device_id")]
        pub device_id: String,
        #[sea_orm(column_name = "device_name")]
        pub device_name: Option<String>,
        #[sea_orm(column_name = "platform")]
        pub platform: String,
        #[sea_orm(column_name = "access_token")]
        pub access_token: String,
        #[sea_orm(column_name = "access_expires_at")]
        pub access_expires_at: DateTime,
        #[sea_orm(column_name = "refresh_token")]
        pub refresh_token: String,
        #[sea_orm(column_name = "refresh_expires_at")]
        pub refresh_expires_at: DateTime,
        #[sea_orm(column_name = "created_at")]
        pub created_at: DateTime,
        #[sea_orm(column_name = "expired_at")]
        pub expired_at: DateTime,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod auth_challenge {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "auth_challenge")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: Uuid,
        #[sea_orm(column_name = "account_id")]
        pub account_id: Uuid,
        #[sea_orm(column_name = "device_id")]
        pub device_id: String,
        #[sea_orm(column_name = "device_name")]
        pub device_name: Option<String>,
        #[sea_orm(column_name = "platform")]
        pub platform: String,
        #[sea_orm(column_name = "ip_address")]
        pub ip_address: Option<String>,
        #[sea_orm(column_name = "user_agent")]
        pub user_agent: Option<String>,
        #[sea_orm(column_name = "step_total")]
        pub step_total: i32,
        #[sea_orm(column_name = "step_remain")]
        pub step_remain: i32,
        #[sea_orm(column_name = "failed_attempts")]
        pub failed_attempts: i32,
        #[sea_orm(column_name = "expired_at")]
        pub expired_at: Option<DateTime>,
        #[sea_orm(column_name = "approved_at")]
        pub approved_at: Option<DateTime>,
        #[sea_orm(column_name = "declined_at")]
        pub declined_at: Option<DateTime>,
        #[sea_orm(column_name = "created_at")]
        pub created_at: DateTime,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}