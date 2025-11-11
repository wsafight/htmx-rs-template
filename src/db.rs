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
static MIGRATIONS: &[MigrationInfo] = &[
    MigrationInfo {
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
    },
    MigrationInfo {
        version: 2,
        sql: r#"
        -- 为users表的name和email字段添加索引，优化搜索性能
        CREATE INDEX IF NOT EXISTS idx_users_name ON users(name);
        CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
        -- 为todos表的completed字段添加索引，优化状态过滤
        CREATE INDEX IF NOT EXISTS idx_todos_completed ON todos(completed);
        -- 为todos表的id字段添加降序索引，优化排序查询
        CREATE INDEX IF NOT EXISTS idx_todos_id_desc ON todos(id DESC);
        "#,
    },
];

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

    // 从环境变量获取连接池配置（用于生产环境调整）
    let max_connections = std::env::var("DB_MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(15); // 增加最大连接数以支持更多并发

    let min_connections = std::env::var("DB_MIN_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3); // 适当增加最小连接数以减少冷启动延迟

    let acquire_timeout = std::env::var("DB_ACQUIRE_TIMEOUT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8); // 增加超时时间以适应高负载情况

    let idle_timeout = std::env::var("DB_IDLE_TIMEOUT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(600); // 延长空闲超时以保持连接热备

    // 创建连接选项
    let options = SqliteConnectOptions::from_str(&database_url)?
        .journal_mode(SqliteJournalMode::Wal) // 使用WAL模式提高并发性能
        .busy_timeout(Duration::from_secs(10)) // 增加busy_timeout以处理并发写入
        .create_if_missing(true)
        .pragma("synchronous", "NORMAL") // 优化写入性能
        .pragma("temp_store", "MEMORY") // 临时表使用内存
        .pragma("cache_size", "-65536"); // 增加缓存大小约512MB

    // 配置连接池
    let pool = SqlitePoolOptions::new()
        .max_connections(max_connections)
        .min_connections(min_connections)
        .acquire_timeout(Duration::from_secs(acquire_timeout))
        .idle_timeout(Duration::from_secs(idle_timeout))
        .max_lifetime(Duration::from_secs(3600)) // 添加最大生命周期，防止连接泄漏
        .connect_with(options)
        .await?;

    tracing::info!(
        "✅ 数据库连接池创建成功 [最大: {}, 最小: {}, 超时: {}s]",
        max_connections,
        min_connections,
        acquire_timeout
    );
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
        // 插入示例用户（20个）
        let users = vec![
            ("张三", "zhangsan@example.com"),
            ("李四", "lisi@example.com"),
            ("王五", "wangwu@example.com"),
            ("赵六", "zhaoliu@example.com"),
            ("孙七", "sunqi@example.com"),
            ("周八", "zhouba@example.com"),
            ("吴九", "wujiu@example.com"),
            ("郑十", "zhengshi@example.com"),
            ("陈一一", "chenyiyi@example.com"),
            ("褚一二", "chuyier@example.com"),
            ("卫一三", "weiyisan@example.com"),
            ("蒋一四", "jiangyisi@example.com"),
            ("沈一五", "shenyiwu@example.com"),
            ("韩一六", "hanyiliu@example.com"),
            ("杨一七", "yangyiqi@example.com"),
            ("朱一八", "zhuyiba@example.com"),
            ("秦一九", "qinyijiu@example.com"),
            ("尤二十", "youershi@example.com"),
            ("许二一", "xueryi@example.com"),
            ("何二二", "heerer@example.com"),
        ];
        let user_count = users.len();

        for (name, email) in users {
            sqlx::query("INSERT INTO users (name, email) VALUES (?, ?)")
                .bind(name)
                .bind(email)
                .execute(&mut *tx)
                .await?;
        }

        tracing::info!("✅ 插入 {} 个用户示例数据", user_count);
    }

    tx.commit().await?;
    Ok(())
}

/// 简化的数据库初始化函数（兼容旧接口）
#[allow(dead_code)]
pub async fn init_db(pool: &SqlitePool) -> Result<(), DbError> {
    run_migrations(pool).await
}
