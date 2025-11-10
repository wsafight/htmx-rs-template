//! 页面路由处理模块
//! 
//! 提供各种页面的渲染功能，包含错误处理和缓存机制

use askama::Template;
use askama_axum::IntoResponse;
use axum::{http::StatusCode, Extension};
use futures::future;
use sqlx::SqlitePool;


// 导入缓存模块
use crate::helpers::cache::{invalidate_cache, get_from_cache, set_to_cache};

// 导入其他模块的类型
use super::todos::Todo;
use super::users::User;

// 定义缓存键常量，避免硬编码
pub const CACHE_KEY_TODOS: &str = "todos";
pub const CACHE_KEY_USERS: &str = "users";
pub const INITIAL_USERS_CACHE_KEY: &str = "initial_users";

// 获取待办事项（带缓存）
async fn get_todos_with_cache(
    pool: &SqlitePool,
) -> Result<(Vec<Todo>, usize, usize), sqlx::Error> {
    // 尝试从缓存获取
    if let Some((todos, completed_count, pending_count)) = get_from_cache(CACHE_KEY_TODOS) {
        return Ok((todos, completed_count, pending_count));
    }

    // 缓存未命中或过期，并行获取待办事项和统计信息
    let (todos, stats) = future::join(
        super::todos::get_todos(pool),
        super::todos::get_stats(pool)
    ).await;
    
    let todos = todos?;
    let stats = stats?;
    
    // 更新缓存，使用显式的过期时间（15分钟）
    set_to_cache(
        CACHE_KEY_TODOS, 
        (todos.clone(), stats.completed_count, stats.pending_count), 
        Some(std::time::Duration::from_secs(900))
    );

    Ok((todos, stats.completed_count, stats.pending_count))
}

// 获取用户列表（带缓存）
async fn get_users_with_cache(pool: &SqlitePool) -> Result<Vec<User>, sqlx::Error> {
    // 尝试从缓存获取
    if let Some(users) = get_from_cache(CACHE_KEY_USERS) {
        return Ok(users);
    }

    // 缓存未命中或过期，从数据库获取
    let users = super::users::get_all_users(pool).await?;

    // 更新缓存，使用显式的过期时间（10分钟）
    set_to_cache(
        CACHE_KEY_USERS, 
        users.clone(), 
        Some(std::time::Duration::from_secs(600))
    );

    Ok(users)
}

// 完整页面模板（首次加载）
#[derive(Template)]
#[template(path = "modules/home/index.html")]
pub struct IndexTemplate;

// 完整页面模板（包含 base.html，用于直接访问）
#[derive(Template)]
#[template(path = "modules/todos/index.html")]
pub struct TodosFullPageTemplate {
    pub todos: Vec<Todo>,
    pub completed_count: usize,
    pub pending_count: usize,
}

#[derive(Template)]
#[template(path = "modules/users/index.html")]
pub struct UsersFullPageTemplate {
    pub users: Vec<User>,
}

// SPA 页面内容片段（不包含 base.html）
#[derive(Template)]
#[template(path = "modules/home/main.html")]
pub struct HomePageTemplate;

#[derive(Template)]
#[template(path = "modules/todos/main.html")]
pub struct TodosPageTemplate {
    pub todos: Vec<Todo>,
    pub completed_count: usize,
    pub pending_count: usize,
}

#[derive(Template)]
#[template(path = "modules/users/main.html")]
pub struct UsersPageTemplate {
    pub users: Vec<User>,
}

/// 首次访问返回完整页面
pub async fn index() -> impl IntoResponse {
    IndexTemplate
}

/// 直接访问 /todos 返回完整页面
pub async fn todos_page(Extension(pool): Extension<SqlitePool>) -> impl IntoResponse {
    match get_todos_with_cache(&pool).await {
        Ok((todos, completed_count, pending_count)) => TodosFullPageTemplate {
            todos,
            completed_count,
            pending_count,
        }
        .into_response(),
        Err(e) => {
            tracing::error!("获取待办事项失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "获取数据失败，请稍后重试",
            )
                .into_response()
        }
    }
}

/// 直接访问 /users 返回完整页面
pub async fn users_page(Extension(pool): Extension<SqlitePool>) -> impl IntoResponse {
    match get_users_with_cache(&pool).await {
        Ok(users) => UsersFullPageTemplate { users }.into_response(),
        Err(e) => {
            tracing::error!("获取用户列表失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "获取数据失败，请稍后重试",
            )
                .into_response()
        }
    }
}

/// SPA 页面内容 - 首页
pub async fn page_home() -> impl IntoResponse {
    HomePageTemplate
}

/// SPA 页面内容 - 待办事项
pub async fn page_todos(Extension(pool): Extension<SqlitePool>) -> impl IntoResponse {
    match get_todos_with_cache(&pool).await {
        Ok((todos, completed_count, pending_count)) => TodosPageTemplate {
            todos,
            completed_count,
            pending_count,
        }
        .into_response(),
        Err(e) => {
            tracing::error!("获取待办事项失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "获取数据失败，请稍后重试",
            )
                .into_response()
        }
    }
}

/// SPA 页面内容 - 用户列表
pub async fn page_users(Extension(pool): Extension<SqlitePool>) -> impl IntoResponse {
    // 使用专门的缓存键存储初始用户列表，避免缓存整个用户列表
    
    // 尝试从缓存获取初始用户列表
    if let Some(users) = get_from_cache(INITIAL_USERS_CACHE_KEY) {
        return UsersPageTemplate { users }.into_response();
    }
    
    // 缓存未命中，从数据库获取前12个用户
    let users = sqlx::query_as::<_, User>(
        "SELECT id, name, email FROM users ORDER BY id LIMIT 12"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();
    
    // 缓存初始用户列表，设置较短的过期时间（5分钟）
    set_to_cache(
        INITIAL_USERS_CACHE_KEY, 
        users.clone(), 
        Some(std::time::Duration::from_secs(300))
    );

    UsersPageTemplate { users }.into_response()
}

// 导出缓存失效函数，供其他模块调用
pub fn invalidate_todo_cache() {
    // 使待办事项缓存失效
    invalidate_cache(CACHE_KEY_TODOS);
}

#[allow(dead_code)]
pub fn invalidate_user_cache() {
    // 使用户缓存失效
    invalidate_cache(CACHE_KEY_USERS);
}