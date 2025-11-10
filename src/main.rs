mod db;
mod routes;

use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::get,
    Extension, Router,
};
use rust_embed::RustEmbed;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(RustEmbed)]
#[folder = "static/"]
struct StaticAssets;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "htmx_rs_template=debug,tower_http=debug,sqlx=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 创建数据库连接池
    tracing::info!("🔧 正在连接数据库...");
    let pool = db::create_pool().await.expect("无法创建数据库连接池");

    // 初始化数据库表
    db::init_db(&pool).await.expect("无法初始化数据库");

    // 插入示例数据
    db::seed_data(&pool).await.expect("无法插入示例数据");

    tracing::info!("✅ 数据库初始化完成");

    let app = Router::new()
        // 首次加载完整页面（支持直接访问）
        .route("/", get(routes::index))
        .route("/todos", get(routes::todos_page))
        .route("/users", get(routes::users_page))
        // SPA 页面内容路由（返回 HTML 片段）
        .route("/page/home", get(routes::page_home))
        .route("/page/todos", get(routes::page_todos))
        .route("/page/users", get(routes::page_users))
        // 待办事项 API
        .route("/todos/create", get(routes::todos::create_form))
        .route("/api/todos", axum::routing::post(routes::todos::create))
        .route("/todos/:id", axum::routing::delete(routes::todos::delete))
        .route(
            "/todos/:id/toggle",
            axum::routing::put(routes::todos::toggle),
        )
        // 用户 API
        .route("/users/search", get(routes::users::search))
        .route("/users/:id/detail", get(routes::users::detail))
        // 模态框
        .route("/modal/example", get(routes::modal::example))
        // 静态文件（嵌入式）
        .route("/static/*path", get(static_handler))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(pool)); // 将数据库连接池注入到所有路由

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::info!(
        "🚀 SPA Server listening on http://{}",
        listener.local_addr().unwrap()
    );
    tracing::info!("📱 Navigate pages without refresh!");
    tracing::info!("💾 SQLite database: app.db");
    axum::serve(listener, app).await.unwrap();
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches("/static/");

    match StaticAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .header(header::CACHE_CONTROL, "public, max-age=31536000")
                .body(Body::from(content.data))
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("404 Not Found"))
            .unwrap(),
    }
}
