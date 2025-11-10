use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    Form,
};
use serde::Deserialize;
use sqlx::SqlitePool;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub completed: bool,
}

#[derive(Template)]
#[template(path = "modules/todos/item.html")]
pub struct TodoItemTemplate {
    pub todo: Todo,
}

#[derive(Template)]
#[template(path = "modules/todos/create_form.html")]
pub struct CreateFormTemplate;

#[derive(Template)]
#[template(path = "modules/todos/stats.html")]
pub struct TodoStatsTemplate {
    pub total_count: usize,
    pub completed_count: usize,
    pub pending_count: usize,
}

#[derive(Deserialize)]
pub struct CreateTodoForm {
    title: String,
}

/// 从数据库获取所有待办事项
pub async fn get_todos(pool: &SqlitePool) -> Result<Vec<Todo>, sqlx::Error> {
    sqlx::query_as::<_, Todo>("SELECT id, title, completed FROM todos ORDER BY id DESC")
        .fetch_all(pool)
        .await
}

/// 获取统计信息
async fn get_stats(pool: &SqlitePool) -> Result<TodoStatsTemplate, sqlx::Error> {
    let todos = get_todos(pool).await?;
    let total_count = todos.len();
    let completed_count = todos.iter().filter(|t| t.completed).count();
    let pending_count = total_count - completed_count;

    Ok(TodoStatsTemplate {
        total_count,
        completed_count,
        pending_count,
    })
}

pub async fn create_form() -> impl IntoResponse {
    CreateFormTemplate
}

pub async fn create(
    Extension(pool): Extension<SqlitePool>,
    Form(form): Form<CreateTodoForm>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Todo>(
        "INSERT INTO todos (title, completed) VALUES (?, 0) RETURNING id, title, completed",
    )
    .bind(&form.title)
    .fetch_one(&pool)
    .await;

    match result {
        Ok(todo) => {
            let stats = get_stats(&pool).await.unwrap_or(TodoStatsTemplate {
                total_count: 0,
                completed_count: 0,
                pending_count: 0,
            });
            let todo_html = TodoItemTemplate { todo }.render().unwrap_or_default();
            let stats_html = stats.render().unwrap_or_default();

            // 返回待办项和统计信息，使用 hx-swap-oob 更新统计区域
            format!(
                "{}<div id=\"todo-stats\" class=\"row mt-4\" hx-swap-oob=\"true\">{}</div>",
                todo_html, stats_html
            )
            .into_response()
        }
        Err(e) => {
            tracing::error!("创建待办失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "创建失败").into_response()
        }
    }
}

pub async fn delete(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM todos WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await;

    match result {
        Ok(_) => {
            let stats = get_stats(&pool).await.unwrap_or(TodoStatsTemplate {
                total_count: 0,
                completed_count: 0,
                pending_count: 0,
            });
            let stats_html = stats.render().unwrap_or_default();

            // 返回空内容（删除当前元素）和更新的统计信息
            format!(
                "<div id=\"todo-stats\" class=\"row mt-4\" hx-swap-oob=\"true\">{}</div>",
                stats_html
            )
            .into_response()
        }
        Err(e) => {
            tracing::error!("删除待办失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn toggle(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    // 切换完成状态
    let result = sqlx::query_as::<_, Todo>(
        "UPDATE todos SET completed = NOT completed WHERE id = ? RETURNING id, title, completed",
    )
    .bind(id)
    .fetch_one(&pool)
    .await;

    match result {
        Ok(todo) => {
            let stats = get_stats(&pool).await.unwrap_or(TodoStatsTemplate {
                total_count: 0,
                completed_count: 0,
                pending_count: 0,
            });
            let todo_html = TodoItemTemplate { todo }.render().unwrap_or_default();
            let stats_html = stats.render().unwrap_or_default();

            // 返回待办项和统计信息
            format!(
                "{}<div id=\"todo-stats\" class=\"row mt-4\" hx-swap-oob=\"true\">{}</div>",
                todo_html, stats_html
            )
            .into_response()
        }
        Err(e) => {
            tracing::error!("切换待办状态失败: {}", e);
            StatusCode::NOT_FOUND.into_response()
        }
    }
}
