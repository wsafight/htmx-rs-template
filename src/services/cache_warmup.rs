//! 缓存预热服务
//!
//! 提供在应用启动时预加载热点数据到缓存的功能，减少冷启动时间和首次请求延迟

use sqlx::{Error as SqlxError, SqlitePool};
use tracing::{info, warn};

// 定义模块内通用的Result类型
type Result<T, E = SqlxError> = std::result::Result<T, E>;

use crate::helpers::cache::set_to_cache;
use crate::routes::pages::{CACHE_KEY_TODOS, CACHE_KEY_USERS, INITIAL_USERS_CACHE_KEY};
use crate::routes::todos::{get_stats, get_todos};
use crate::routes::users::get_all_users;

/// 预加载所有热点数据到缓存
/// 这个函数应该在应用启动时异步调用
pub async fn warmup_all_caches(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    info!("开始缓存预热...");

    // 并行预热多个缓存
    let results = tokio::join!(
        warmup_todos_cache(pool),
        warmup_users_cache(pool),
        warmup_initial_users_cache(pool)
    );

    // 统计预热结果
    let mut success_count = 0;
    let mut failure_count = 0;

    if results.0.is_ok() {
        success_count += 1;
    } else {
        failure_count += 1;
    }

    if results.1.is_ok() {
        success_count += 1;
    } else {
        failure_count += 1;
    }

    if results.2.is_ok() {
        success_count += 1;
    } else {
        failure_count += 1;
    }

    info!(
        "缓存预热完成: 成功 {}, 失败 {}",
        success_count, failure_count
    );
    Ok(())
}

/// 预热待办事项缓存
async fn warmup_todos_cache(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    info!("预热待办事项缓存...");

    // 获取待办事项和统计信息
    let (todos, stats) = tokio::join!(get_todos(pool), get_stats(pool));

    let todos = todos?;
    let stats = stats?;

    // 设置缓存，过期时间15分钟
    set_to_cache(
        CACHE_KEY_TODOS,
        (todos, stats.completed_count, stats.pending_count),
        Some(std::time::Duration::from_secs(900)),
    );

    info!("待办事项缓存预热成功");
    Ok(())
}

/// 预热用户列表缓存
async fn warmup_users_cache(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    info!("预热用户列表缓存...");

    // 获取所有用户
    let users = get_all_users(pool).await?;

    // 设置缓存，过期时间10分钟
    set_to_cache(
        CACHE_KEY_USERS,
        users,
        Some(std::time::Duration::from_secs(600)),
    );

    info!("用户列表缓存预热成功");
    Ok(())
}

/// 预热初始用户列表缓存（前12个用户）
async fn warmup_initial_users_cache(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    info!("预热初始用户列表缓存...");

    // 导入User类型
    use crate::routes::users::User;

    // 获取前12个用户
    let users = sqlx::query_as::<_, User>("SELECT id, name, email FROM users ORDER BY id LIMIT 12")
        .fetch_all(pool)
        .await?;

    // 设置缓存，过期时间5分钟
    set_to_cache(
        INITIAL_USERS_CACHE_KEY,
        users,
        Some(std::time::Duration::from_secs(300)),
    );

    info!("初始用户列表缓存预热成功");
    Ok(())
}

/// 定期刷新缓存的后台任务
/// 可以在应用中启动一个独立的任务来定期执行
pub async fn start_cache_refresh_task(pool: SqlitePool) {
    let refresh_interval = std::time::Duration::from_secs(300); // 5分钟刷新一次

    info!("启动缓存自动刷新任务，间隔: {:?}", refresh_interval);

    loop {
        tokio::time::sleep(refresh_interval).await;

        info!("开始自动刷新缓存...");

        // 执行缓存预热
        let result = warmup_all_caches(&pool).await;

        if result.is_err() {
            warn!("缓存自动刷新失败: {:?}", result);
        }
    }
}
