pub mod modal;
pub mod todos;
pub mod users;

use askama::Template;
use askama_axum::IntoResponse;
use axum::Extension;
use sqlx::SqlitePool;

// 完整页面模板（首次加载）
#[derive(Template)]
#[template(path = "modules/home/index.html")]
pub struct IndexTemplate;

// 完整页面模板（包含 base.html，用于直接访问）
#[derive(Template)]
#[template(path = "modules/todos/index.html")]
pub struct TodosFullPageTemplate {
    pub todos: Vec<todos::Todo>,
    pub completed_count: usize,
    pub pending_count: usize,
}

#[derive(Template)]
#[template(path = "modules/users/index.html")]
pub struct UsersFullPageTemplate {
    pub users: Vec<users::User>,
}


// SPA 页面内容片段（不包含 base.html）
#[derive(Template)]
#[template(path = "modules/home/main.html")]
pub struct HomePageTemplate;

#[derive(Template)]
#[template(path = "modules/todos/main.html")]
pub struct TodosPageTemplate {
    pub todos: Vec<todos::Todo>,
    pub completed_count: usize,
    pub pending_count: usize,
}

#[derive(Template)]
#[template(path = "modules/users/main.html")]
pub struct UsersPageTemplate {
    pub users: Vec<users::User>,
}


// 首次访问返回完整页面
pub async fn index() -> impl IntoResponse {
    IndexTemplate
}

// 直接访问 /todos 返回完整页面
pub async fn todos_page(Extension(pool): Extension<SqlitePool>) -> impl IntoResponse {
    let todos = todos::get_todos(&pool).await.unwrap_or_default();
    let completed_count = todos.iter().filter(|t| t.completed).count();
    let pending_count = todos.iter().filter(|t| !t.completed).count();

    TodosFullPageTemplate {
        todos,
        completed_count,
        pending_count,
    }
}

// 直接访问 /users 返回完整页面
pub async fn users_page(Extension(pool): Extension<SqlitePool>) -> impl IntoResponse {
    let users = users::get_all_users(&pool).await.unwrap_or_default();
    UsersFullPageTemplate { users }
}

// SPA 页面内容
pub async fn page_home() -> impl IntoResponse {
    HomePageTemplate
}

pub async fn page_todos(Extension(pool): Extension<SqlitePool>) -> impl IntoResponse {
    let todos = todos::get_todos(&pool).await.unwrap_or_default();
    let completed_count = todos.iter().filter(|t| t.completed).count();
    let pending_count = todos.iter().filter(|t| !t.completed).count();

    TodosPageTemplate {
        todos,
        completed_count,
        pending_count,
    }
}

pub async fn page_users(Extension(pool): Extension<SqlitePool>) -> impl IntoResponse {
    let users = users::get_all_users(&pool).await.unwrap_or_default();
    UsersPageTemplate { users }
}
