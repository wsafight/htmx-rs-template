//! 页面路由处理模块
//!
//! 提供各种页面的渲染功能，包含错误处理和缓存机制

use askama::Template;
use askama_axum::IntoResponse;
use axum::{http::StatusCode, Extension};
use sqlx::SqlitePool;

use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

// 导入其他模块的类型
use super::todos::Todo;
use super::users::User;

// 缓存条目结构
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    data: T,
    timestamp: Instant,
}

// 内存缓存管理器
pub struct CacheManager {
    todo_cache: RwLock<Option<CacheEntry<(Vec<Todo>, usize, usize)>>>,
    user_cache: RwLock<Option<CacheEntry<Vec<User>>>>,
    cache_duration: Duration,
}

impl CacheManager {
    fn new() -> Self {
        Self {
            todo_cache: RwLock::new(None),
            user_cache: RwLock::new(None),
            cache_duration: Duration::from_secs(60), // 缓存1分钟
        }
    }

    // 获取待办事项（带缓存）
    async fn get_todos_with_cache(
        &self,
        pool: &SqlitePool,
    ) -> Result<(Vec<Todo>, usize, usize), sqlx::Error> {
        // 尝试从缓存获取
        if let Some(cache_entry) = &*self.todo_cache.read().unwrap() {
            if Instant::now() - cache_entry.timestamp < self.cache_duration {
                return Ok(cache_entry.data.clone());
            }
        }

        // 缓存未命中或过期，从数据库获取
        let todos = super::todos::get_todos(pool).await?;
        let completed_count = todos.iter().filter(|t| t.completed).count();
        let pending_count = todos.iter().filter(|t| !t.completed).count();

        // 更新缓存
        *self.todo_cache.write().unwrap() = Some(CacheEntry {
            data: (todos.clone(), completed_count, pending_count),
            timestamp: Instant::now(),
        });

        Ok((todos, completed_count, pending_count))
    }

    // 获取用户列表（带缓存）
    pub async fn get_users_with_cache(&self, pool: &SqlitePool) -> Result<Vec<User>, sqlx::Error> {
        // 尝试从缓存获取
        if let Some(cache_entry) = &*self.user_cache.read().unwrap() {
            if Instant::now() - cache_entry.timestamp < self.cache_duration {
                return Ok(cache_entry.data.clone());
            }
        }

        // 缓存未命中或过期，从数据库获取
        let users = super::users::get_all_users(pool).await?;

        // 更新缓存
        *self.user_cache.write().unwrap() = Some(CacheEntry {
            data: users.clone(),
            timestamp: Instant::now(),
        });

        Ok(users)
    }

    // 清除待办事项缓存
    fn invalidate_todo_cache(&self) {
        *self.todo_cache.write().unwrap() = None;
    }

    // 清除用户缓存
    fn invalidate_user_cache(&self) {
        *self.user_cache.write().unwrap() = None;
    }
}

// 全局缓存管理器实例
lazy_static::lazy_static! {
    pub static ref CACHE_MANAGER: Arc<CacheManager> = Arc::new(CacheManager::new());
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
    match CACHE_MANAGER.get_todos_with_cache(&pool).await {
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
    match CACHE_MANAGER.get_users_with_cache(&pool).await {
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
    match CACHE_MANAGER.get_todos_with_cache(&pool).await {
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
    // 获取前12个用户用于初始显示
    let users = sqlx::query_as::<_, User>("SELECT id, name, email FROM users ORDER BY id LIMIT 12")
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    UsersPageTemplate { users }.into_response()
}

// 导出缓存失效函数，供其他模块调用
pub fn invalidate_todo_cache() {
    CACHE_MANAGER.invalidate_todo_cache();
}

pub fn invalidate_user_cache() {
    CACHE_MANAGER.invalidate_user_cache();
}
