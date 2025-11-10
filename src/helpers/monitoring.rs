//! 监控和运维功能模块
//! 
//! 提供健康检查、性能指标收集和API文档功能

use axum::{extract::State, http::StatusCode, response::IntoResponse, Router};
use metrics::{counter, gauge, increment_counter};
use metrics_exporter_prometheus::PrometheusBuilder;
use serde::Serialize;
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::Instant;

use crate::helpers::config::AppConfig;

/// 健康检查响应
#[derive(Serialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub version: String,
    pub uptime: u64,
    pub database: String,
}

/// 应用状态，包含启动时间和数据库连接池
#[derive(Clone)]
pub struct AppState {
    pub start_time: Instant,
    pub pool: SqlitePool,
    #[allow(dead_code)]
    pub config: Arc<AppConfig>,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new(pool: SqlitePool, config: Arc<AppConfig>) -> Self {
        Self {
            start_time: Instant::now(),
            pool,
            config,
        }
    }

    /// 获取应用运行时间（秒）
    pub fn uptime(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}

/// 初始化指标收集器
pub fn init_metrics() {
    // 设置 Prometheus 指标收集器
    let builder = PrometheusBuilder::new();
    builder
        .install()
        .expect("Failed to install Prometheus metrics exporter");

    // 初始化一些计数器
    counter!("http_requests_total", 0);
    gauge!("app_uptime_seconds", 0.0);
}

/// 健康检查处理器
pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    // 增加健康检查计数
    increment_counter!("http_requests_total");

    // 更新运行时间指标
    gauge!("app_uptime_seconds", state.uptime() as f64);

    // 检查数据库连接
    let db_status = match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => "ok",
        Err(e) => {
            tracing::error!("数据库健康检查失败: {}", e);
            "error"
        }
    };

    // 构建健康检查响应
    let response = HealthCheckResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: state.uptime(),
        database: db_status.to_string(),
    };

    // 返回 JSON 响应
    (StatusCode::OK, axum::Json(response)).into_response()
}

/// 指标收集中间件
pub async fn metrics_middleware(
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> impl IntoResponse {
    let start = Instant::now();
    let _path = req.uri().path().to_string();
    let _method = req.method().to_string();

    // 处理请求
    let response = next.run(req).await;

    // 计算处理时间
    let duration = start.elapsed();
    let _status = response.status().as_u16().to_string();

    // 记录指标
    increment_counter!("http_requests_total");
    counter!(
        "http_request_duration_seconds",
        duration.as_secs_f64() as u64
    );

    response
}

/// 创建监控路由
pub fn create_monitoring_routes(state: AppState) -> Router {
    use axum::routing::get;

    // 创建路由
    Router::new()
        .route("/health", get(health_check))
        .with_state(state)
}