//! 页面路由处理模块
//! 
//! 提供各种页面的渲染功能

use askama::Template;
use askama_axum::IntoResponse;
use axum::Extension;
use sqlx::SqlitePool;

// 导入其他模块的类型
use super::todos::Todo;
use super::users::User;

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
    let todos = super::todos::get_todos(&pool).await.unwrap_or_default();
    let completed_count = todos.iter().filter(|t| t.completed).count();
    let pending_count = todos.iter().filter(|t| !t.completed).count();

    TodosFullPageTemplate {
        todos,
        completed_count,
        pending_count,
    }
}

/// 直接访问 /users 返回完整页面
pub async fn users_page(Extension(pool): Extension<SqlitePool>) -> impl IntoResponse {
    let users = super::users::get_all_users(&pool).await.unwrap_or_default();
    UsersFullPageTemplate { users }
}

/// SPA 页面内容 - 首页
pub async fn page_home() -> impl IntoResponse {
    HomePageTemplate
}

/// SPA 页面内容 - 待办事项
pub async fn page_todos(Extension(pool): Extension<SqlitePool>) -> impl IntoResponse {
    let todos = super::todos::get_todos(&pool).await.unwrap_or_default();
    let completed_count = todos.iter().filter(|t| t.completed).count();
    let pending_count = todos.iter().filter(|t| !t.completed).count();

    TodosPageTemplate {
        todos,
        completed_count,
        pending_count,
    }
}

/// SPA 页面内容 - 用户列表
pub async fn page_users(Extension(pool): Extension<SqlitePool>) -> impl IntoResponse {
    let users = super::users::get_all_users(&pool).await.unwrap_or_default();
    UsersPageTemplate { users }
}