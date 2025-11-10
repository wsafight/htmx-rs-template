mod db;
mod routes;

use axum::{
    routing::get,
    Extension, Router,
};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
        // 官网首页
        .route("/", get(routes::official::index))
        // /app 开头 - 返回完整 HTML 页面
        .route("/app", get(routes::pages::index))
        .route("/app/todos", get(routes::pages::todos_page))
        .route("/app/users", get(routes::pages::users_page))
        // /block 开头 - 返回 HTML 片段
        .route("/block/home", get(routes::pages::page_home))
        .route("/block/todos", get(routes::pages::page_todos))
        .route("/block/users", get(routes::pages::page_users))
        .route("/block/todos/create-form", get(routes::todos::create_form))
        .route("/block/users/search", get(routes::users::search))
        .route("/block/users/:id/detail", get(routes::users::detail))
        .route("/block/modal/example", get(routes::modal::example))
        // /api 开头 - 返回 JSON 或执行操作后返回 HTML 片段
        .route("/api/todos", axum::routing::post(routes::todos::create))
        .route(
            "/api/todos/:id",
            axum::routing::delete(routes::todos::delete),
        )
        .route(
            "/api/todos/:id/toggle",
            axum::routing::put(routes::todos::toggle),
        )
        // 静态文件（嵌入式）
        .route("/static/*path", get(routes::static_assets::static_handler))
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
