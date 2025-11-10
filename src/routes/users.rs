use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Extension, Path, Query};
use axum::http::StatusCode;
use serde::Deserialize;
use sqlx::SqlitePool;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
}

#[derive(Template)]
#[template(path = "users/search_results.html")]
pub struct UserSearchResultsTemplate {
    pub users: Vec<User>,
    pub query: String,
}

#[derive(Template)]
#[template(path = "users/detail.html")]
pub struct UserDetailTemplate {
    pub user: User,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
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

    let users = if query.is_empty() {
        get_all_users(&pool).await.unwrap_or_default()
    } else {
        // 使用 LIKE 进行模糊搜索
        let search_pattern = format!("%{}%", query);
        sqlx::query_as::<_, User>(
            "SELECT id, name, email FROM users WHERE name LIKE ? OR email LIKE ? ORDER BY id",
        )
        .bind(&search_pattern)
        .bind(&search_pattern)
        .fetch_all(&pool)
        .await
        .unwrap_or_default()
    };

    UserSearchResultsTemplate { users, query }
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
