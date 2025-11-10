use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions},
    Error as SqlxError, Transaction,
};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use thiserror::Error;

/// 数据库操作错误类型
#[derive(Error, Debug)]
pub enum DbError {
    #[error("数据库连接错误: {0}")]
    Connection(#[from] SqlxError),
    #[error("数据库迁移错误: {0}")]
    Migration(String),
    #[error("事务操作错误: {0}")]
    Transaction(String),
}

/// 数据库迁移信息
#[derive(Debug, Clone)]
pub struct MigrationInfo {
    pub version: i64,
    pub sql: &'static str,
}

// 定义数据库迁移
static MIGRATIONS: &[MigrationInfo] = &[MigrationInfo {
    version: 1,
    sql: r#"
        CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            completed BOOLEAN NOT NULL DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#,
}];

/// 获取可执行文件所在目录的数据库路径
fn get_default_db_path() -> String {
    // 获取当前可执行文件的路径
    let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));

    // 获取可执行文件所在的目录
    let exe_dir = exe_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));

    // 在可执行文件目录下创建数据库文件
    let db_path = exe_dir.join("app.db");

    format!("sqlite://{}?mode=rwc", db_path.display())
}

/// 创建数据库连接池
pub async fn create_pool() -> Result<SqlitePool, DbError> {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| get_default_db_path());

    tracing::info!("📂 数据库路径: {}", database_url);

    // 创建连接选项
    let options = SqliteConnectOptions::from_str(&database_url)?
        .journal_mode(SqliteJournalMode::Wal) // 使用WAL模式提高并发性能
        .busy_timeout(Duration::from_secs(5))
        .create_if_missing(true);

    // 配置连接池
    let pool = SqlitePoolOptions::new()
        .max_connections(10) // 增加最大连接数
        .min_connections(2) // 保持最小连接数
        .acquire_timeout(Duration::from_secs(5)) // 增加超时时间
        .idle_timeout(Duration::from_secs(300)) // 设置空闲超时
        .connect_with(options)
        .await?;

    tracing::info!("✅ 数据库连接池创建成功");
    Ok(pool)
}

/// 执行结构化的数据库迁移
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), DbError> {
    let mut tx = start_transaction(pool).await?;

    // 确保schema_migrations表存在
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY, applied_at DATETIME DEFAULT CURRENT_TIMESTAMP)"
    )
    .execute(&mut *tx)
    .await?;

    // 获取最后应用的迁移版本
    let last_version: Option<i64> =
        sqlx::query_scalar("SELECT MAX(version) FROM schema_migrations")
            .fetch_optional(&mut *tx)
            .await?;

    let last_applied = last_version.unwrap_or(0);

    // 应用未应用的迁移
    let mut applied = 0;
    for migration in MIGRATIONS {
        if migration.version > last_applied {
            tracing::info!("应用数据库迁移版本: {}", migration.version);

            sqlx::query(migration.sql)
                .execute(&mut *tx)
                .await
                .map_err(|e| DbError::Migration(format!("版本 {}: {}", migration.version, e)))?;

            // 记录迁移
            sqlx::query("INSERT INTO schema_migrations (version) VALUES (?)")
                .bind(migration.version)
                .execute(&mut *tx)
                .await?;

            applied += 1;
        }
    }

    tx.commit().await?;

    tracing::info!("✅ 数据库迁移完成，应用了 {} 个迁移", applied);
    Ok(())
}

/// 开始数据库事务
pub async fn start_transaction(
    pool: &SqlitePool,
) -> Result<Transaction<'_, sqlx::Sqlite>, DbError> {
    pool.begin()
        .await
        .map_err(|e| DbError::Transaction(e.to_string()))
}

/// 插入示例数据
pub async fn seed_data(pool: &SqlitePool) -> Result<(), DbError> {
    let mut tx = start_transaction(pool).await?;

    // 检查是否已有数据
    let todo_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM todos")
        .fetch_one(&mut *tx)
        .await?;

    if todo_count == 0 {
        // 插入示例待办事项
        sqlx::query("INSERT INTO todos (title, completed) VALUES (?, ?)")
            .bind("学习 Rust")
            .bind(false)
            .execute(&mut *tx)
            .await?;

        sqlx::query("INSERT INTO todos (title, completed) VALUES (?, ?)")
            .bind("学习 HTMX")
            .bind(false)
            .execute(&mut *tx)
            .await?;

        sqlx::query("INSERT INTO todos (title, completed) VALUES (?, ?)")
            .bind("构建 Web 应用")
            .bind(true)
            .execute(&mut *tx)
            .await?;

        tracing::info!("✅ 插入待办事项示例数据");
    }

    // 检查用户数据
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&mut *tx)
        .await?;

    if user_count == 0 {
        // 插入示例用户
        let users = vec![
            ("张三", "zhangsan@example.com"),
            ("李四", "lisi@example.com"),
            ("王五", "wangwu@example.com"),
            ("赵六", "zhaoliu@example.com"),
        ];

        for (name, email) in users {
            sqlx::query("INSERT INTO users (name, email) VALUES (?, ?)")
                .bind(name)
                .bind(email)
                .execute(&mut *tx)
                .await?;
        }

        tracing::info!("✅ 插入用户示例数据");
    }

    tx.commit().await?;
    Ok(())
}

/// 简化的数据库初始化函数（兼容旧接口）
pub async fn init_db(pool: &SqlitePool) -> Result<(), DbError> {
    run_migrations(pool).await
}
