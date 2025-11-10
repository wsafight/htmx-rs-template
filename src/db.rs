use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::PathBuf;
use std::time::Duration;

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
pub async fn create_pool() -> Result<SqlitePool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| get_default_db_path());

    tracing::info!("📂 数据库路径: {}", database_url);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await?;

    Ok(pool)
}

/// 初始化数据库（创建表）
pub async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // 创建待办事项表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            completed BOOLEAN NOT NULL DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    // 创建用户表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    tracing::info!("✅ 数据库表初始化完成");

    Ok(())
}

/// 插入示例数据
pub async fn seed_data(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // 检查是否已有数据
    let todo_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM todos")
        .fetch_one(pool)
        .await?;

    if todo_count == 0 {
        // 插入示例待办事项
        sqlx::query("INSERT INTO todos (title, completed) VALUES (?, ?)")
            .bind("学习 Rust")
            .bind(false)
            .execute(pool)
            .await?;

        sqlx::query("INSERT INTO todos (title, completed) VALUES (?, ?)")
            .bind("学习 HTMX")
            .bind(false)
            .execute(pool)
            .await?;

        sqlx::query("INSERT INTO todos (title, completed) VALUES (?, ?)")
            .bind("构建 Web 应用")
            .bind(true)
            .execute(pool)
            .await?;

        tracing::info!("✅ 插入待办事项示例数据");
    }

    // 检查用户数据
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
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
                .execute(pool)
                .await?;
        }

        tracing::info!("✅ 插入用户示例数据");
    }

    Ok(())
}
