mod config;
mod db;
mod monitoring;
mod routes;
mod security;

use axum::{middleware, routing::get, Extension, Router};
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // 加载配置
    let config = &config::CONFIG;

    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "htmx_rs_template={},tower_http=debug,sqlx=info",
                    config.log_level
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 创建数据库连接池
    tracing::info!("🔧 正在连接数据库...");
    let pool = match db::create_pool().await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!(
                "❌ 无法创建数据库连接池: {}",
                security::sanitize_log_message(&e.to_string())
            );
            std::process::exit(1);
        }
    };

    // 初始化数据库表和运行迁移
    if let Err(e) = db::run_migrations(&pool).await {
        tracing::error!(
            "❌ 数据库迁移失败: {}",
            security::sanitize_log_message(&e.to_string())
        );
        std::process::exit(1);
    }

    // 插入示例数据
    if let Err(e) = db::seed_data(&pool).await {
        tracing::warn!(
            "⚠️  示例数据插入失败: {}",
            security::sanitize_log_message(&e.to_string())
        );
    }

    tracing::info!("✅ 数据库初始化完成");

    // 初始化监控指标
    monitoring::init_metrics();

    // 创建应用状态
    let app_state = monitoring::AppState::new(pool.clone(), Arc::new((*config).clone()));

    // 创建监控路由
    let monitoring_routes = monitoring::create_monitoring_routes(app_state.clone());

    // 配置中间件
    let cors_origins: Vec<_> = config
        .security
        .cors_allow_origins
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();

    let middleware_stack = ServiceBuilder::new()
        // 跟踪请求
        .layer(middleware::from_fn(monitoring::metrics_middleware))
        .layer(TraceLayer::new_for_http())
        // CORS 配置
        .layer(
            CorsLayer::new()
                .allow_origin(cors_origins)
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::PUT,
                    axum::http::Method::DELETE,
                ])
                .allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::ACCEPT])
                .allow_credentials(true),
        )
        // 数据库连接池
        .layer(Extension(pool));

    // 注意：tower-http 0.6版本的compression API已更改，如需添加压缩功能，
    // 请使用以下方式导入和配置：
    // use tower_http::compression::CompressionLayer;
    // .layer(CompressionLayer::new())

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
        // 监控路由
        .merge(monitoring_routes)
        // 应用中间件栈
        .layer(middleware_stack);

    // 绑定地址
    let listener = match tokio::net::TcpListener::bind(config.server.server_addr()).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!(
                "❌ 无法绑定到地址 {}: {}",
                config.server.server_addr(),
                security::sanitize_log_message(&e.to_string())
            );
            std::process::exit(1);
        }
    };

    tracing::info!(
        "🚀 SPA Server listening on http://{}",
        listener.local_addr().unwrap()
    );
    tracing::info!("📱 Navigate pages without refresh!");
    tracing::info!("💾 SQLite database: app.db");
    tracing::info!("🌐 环境: {}", config.environment);

    // 启动服务器，支持优雅关闭
    match axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal(
            config.server.graceful_shutdown_timeout_seconds,
        ))
        .await
    {
        Ok(_) => tracing::info!("✅ 服务器已正常关闭"),
        Err(e) => tracing::error!(
            "❌ 服务器错误: {}",
            security::sanitize_log_message(&e.to_string())
        ),
    }
}

/// 处理优雅关闭信号
async fn shutdown_signal(timeout_seconds: u64) {
    // 等待中断信号
    let ctrl_c = async {
        signal::ctrl_c().await.expect("无法捕获中断信号");
        tracing::info!("收到 CTRL+C 信号，正在关闭服务器...");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("无法捕获终止信号")
            .recv()
            .await;
        tracing::info!("收到终止信号，正在关闭服务器...");
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    // 等待任一信号
    tokio::select! {
        () = ctrl_c => tracing::info!("收到 CTRL+C 信号，正在关闭服务器..."),
        () = terminate => tracing::info!("收到终止信号，正在关闭服务器..."),
    }

    // 等待指定的超时时间后强制关闭
    let timeout = Duration::from_secs(timeout_seconds);
    tokio::time::sleep(timeout).await;
    tracing::info!("超时 {} 秒，强制关闭服务器", timeout_seconds);
}
