use crate::{models::Stats, static_handler::serve_static, LandingConfig};
use askama::Template;
use askama_axum::IntoResponse;
use axum::{routing::get, Router};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    title: String,
    subtitle: String,
    features: Vec<crate::Feature>,
}

#[derive(Template)]
#[template(path = "stats.html")]
struct StatsTemplate {
    user_count: u64,
    project_count: u64,
    satisfaction: u64,
}

/// 首页处理器
async fn index(
    axum::extract::State(config): axum::extract::State<LandingConfig>,
) -> impl IntoResponse {
    IndexTemplate {
        title: config.title,
        subtitle: config.subtitle,
        features: config.features,
    }
}

/// 统计数据处理器
async fn stats() -> impl IntoResponse {
    // 这里可以从数据库获取真实数据
    let stats = Stats::default();

    StatsTemplate {
        user_count: stats.user_count,
        project_count: stats.project_count,
        satisfaction: stats.satisfaction,
    }
}

/// 创建路由
pub fn create_routes(config: LandingConfig) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/stats", get(stats))
        .route("/static/*path", get(serve_static))
        .with_state(config)
}
