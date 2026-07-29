//! 数据库模块
//! 
//! 功能:
//! - MySQL 连接池管理
//! - 数据库迁移
//! - 基础 CRUD 操作

use sqlx::MySqlPool;
use tracing::info;

/// 数据库连接池
pub struct DatabasePool {
    pub pool: MySqlPool,
}

impl DatabasePool {
    /// 创建新的数据库连接池
    pub async fn new(dsn: &str) -> Result<Self, anyhow::Error> {
        info
        
!("正在连接到数据库: {}", dsn);        let pool = MySqlPool::connect(dsn).await?;
        
        // 验证连接
        pool.ping().await?;
        
        info!("数据库连接成功");
        
        Ok(Self { pool })
    }
    
    /// 获取连接池引用
    pub fn pool(&self) -> &MySqlPool {
        &self.pool
    }
    
    /// 运行数据库迁移
    pub async fn run_migrations(&self, migration_dir: &str) -> Result<(), anyhow::Error> {
        info!("正在运行数据库迁移: {}", migration_dir);
        // TODO: 实现迁移逻辑
        Ok(())
    }
}


/// 数据库配置pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub max_connections: u32,
}

impl DbConfig {
    /// 生成 DSN
    pub fn to_dsn(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            host: std::env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("DB_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3306),
            database: std::env::var("DB_NAME").unwrap_or_else(|_| "solsynth".to_string()),
            username: std::env::var("DB_USER").unwrap_or_else(|_| "root".to_string()),
            password: std::env::var("DB_PASS").unwrap_or_else(|_| "".to_string()),
            max_connections: 10,
        }
    }
}