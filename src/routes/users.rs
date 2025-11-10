use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Extension, Path, Query};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
}

#[derive(Template)]
#[template(path = "modules/users/search_results.html")]
pub struct UserSearchResultsTemplate {
    pub users: Vec<User>,
    pub query: String,
    pub pagination: Pagination,
    pub start_item: i64,
    pub end_item: i64,
    pub base_url: String,
    pub target: String,
}

#[derive(Template)]
#[template(path = "modules/users/detail.html")]
pub struct UserDetailTemplate {
    pub user: User,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Pagination {
    pub current_page: i64,
    pub per_page: i64,
    pub total: i64,
    pub total_pages: i64,
    pub has_prev: bool,
    pub has_next: bool,
}

/// 从数据库获取所有用户
pub async fn get_all_users(pool: &SqlitePool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT id, name, email FROM users ORDER BY id")
        .fetch_all(pool)
        .await
}

pub async fn search(
    Extension(pool): Extension<SqlitePool>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    let query = params.q.unwrap_or_default();
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(12).clamp(1, 100);
    let offset = (page - 1) * per_page;

    // 获取总数
    let total: i64 = if query.is_empty() {
        sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&pool)
            .await
            .unwrap_or(0)
    } else {
        let search_pattern = format!("%{}%", query);
        sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE name LIKE ? OR email LIKE ?")
            .bind(&search_pattern)
            .bind(&search_pattern)
            .fetch_one(&pool)
            .await
            .unwrap_or(0)
    };

    // 获取分页数据
    let users = if query.is_empty() {
        sqlx::query_as::<_, User>("SELECT id, name, email FROM users ORDER BY id LIMIT ? OFFSET ?")
            .bind(per_page)
            .bind(offset)
            .fetch_all(&pool)
            .await
            .unwrap_or_default()
    } else {
        let search_pattern = format!("%{}%", query);
        sqlx::query_as::<_, User>(
            "SELECT id, name, email FROM users WHERE name LIKE ? OR email LIKE ? ORDER BY id LIMIT ? OFFSET ?",
        )
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&pool)
        .await
        .unwrap_or_default()
    };

    // 计算分页信息
    let total_pages = (total as f64 / per_page as f64).ceil() as i64;
    let pagination = Pagination {
        current_page: page,
        per_page,
        total,
        total_pages,
        has_prev: page > 1,
        has_next: page < total_pages,
    };

    // 计算显示范围
    let start_item = (page - 1) * per_page + 1;
    let end_item = start_item - 1 + users.len() as i64;

    UserSearchResultsTemplate {
        users,
        query,
        pagination,
        start_item,
        end_item,
        base_url: "/block/users/search".to_string(),
        target: "#search-results".to_string(),
    }
}

pub async fn detail(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, User>("SELECT id, name, email FROM users WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await;

    match result {
        Ok(user) => UserDetailTemplate { user }.into_response(),
        Err(e) => {
            tracing::error!("获取用户详情失败: {}", e);
            (StatusCode::NOT_FOUND, "用户不存在").into_response()
        }
    }
}
