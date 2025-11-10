//! 监控和运维功能模块
//! 
//! 提供健康检查、性能指标收集和API文档功能

use axum::{extract::State, http::StatusCode, response::IntoResponse, Router};
use metrics::{counter, gauge, histogram, increment_counter};
use metrics_exporter_prometheus::PrometheusBuilder;
use serde::Serialize;
use sqlx::{SqlitePool, Error as SqlxError};
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

    // 初始化HTTP请求指标
    counter!("http_requests_total", 0);
    gauge!("app_uptime_seconds", 0.0);
    histogram!("http_request_duration_seconds", 0.0);
    counter!("http_requests_errors_total", 0);
    
    // 初始化数据库指标
    counter!("db_queries_total", 0);
    histogram!("db_query_duration_seconds", 0.0);
    counter!("db_queries_errors_total", 0);
    gauge!("db_connections_active", 0.0);
    gauge!("db_connections_idle", 0.0);
    
    // 初始化缓存指标
    counter!("cache_hits_total", 0);
    counter!("cache_misses_total", 0);
    counter!("cache_sets_total", 0);
    counter!("cache_invalidations_total", 0);
    gauge!("cache_size_items", 0.0);
    
    // 初始化业务指标
    gauge!("todos_count_total", 0.0);
    gauge!("todos_count_completed", 0.0);
    gauge!("users_count_total", 0.0);
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
    let path = req.uri().path().to_string();
    let method = req.method().to_string();

    // 处理请求
    let response = next.run(req).await;

    // 计算处理时间
    let duration = start.elapsed();
    let status = response.status().as_u16().to_string();
    
    // 根据状态码分类记录请求
    if status.starts_with('2') {
        increment_counter!("http_requests_total", "status" => status.clone(), "method" => method.clone(), "path" => path.clone());
    } else {
        increment_counter!("http_requests_errors_total", "status" => status.clone(), "method" => method.clone(), "path" => path.clone());
    }
    
    // 使用histogram记录请求时间分布
    histogram!("http_request_duration_seconds", duration.as_secs_f64(), 
        "status" => status, 
        "method" => method, 
        "path" => path
    );

    response
}

/// 创建监控路由
pub fn create_monitoring_routes(state: AppState) -> Router {
    use axum::routing::get;

    // 创建路由
    Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        .with_state(state)
}

/// 指标处理器 - 暴露Prometheus指标
pub async fn metrics_handler() -> impl IntoResponse {
    // 为了简化，我们返回一个简单的文本响应
    // 注意：在实际生产环境中，需要正确配置metrics_exporter_prometheus
    // 来支持通过HTTP端点暴露指标
    let metrics_text = "# 性能指标暴露端点\n# 请确保正确配置了metrics_exporter_prometheus库\n";
    (
        StatusCode::OK,
        axum::response::Response::builder()
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(axum::body::Body::from(metrics_text))
            .unwrap()
    ).into_response()
}

/// 数据库查询监控帮助函数
pub async fn track_db_query<T, F>(query_name: &str, f: F) -> std::result::Result<T, sqlx::Error>
where
    F: std::future::Future<Output = std::result::Result<T, sqlx::Error>>,
{
    // 增加查询计数
    increment_counter!("db_queries_total", "query" => query_name.to_string());
    
    // 记录查询时间
    let start = Instant::now();
    
    match f.await {
        Ok(result) => {
            // 成功时记录指标
            histogram!("db_query_duration_seconds", start.elapsed().as_secs_f64(), 
                "query" => query_name.to_string(),
                "status" => "success"
            );
            Ok(result)
        },
        Err(e) => {
            // 失败时记录指标
            increment_counter!("db_queries_errors_total", "query" => query_name.to_string());
            histogram!("db_query_duration_seconds", start.elapsed().as_secs_f64(), 
                "query" => query_name.to_string(),
                "status" => "error"
            );
            Err(e)
        }
    }
}