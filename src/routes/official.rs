use askama::Template;
use askama_axum::IntoResponse;

// 官网首页模板
#[derive(Template)]
#[template(path = "official/index.html")]
pub struct OfficialIndexTemplate;

// 官网首页路由处理
pub async fn index() -> impl IntoResponse {
    OfficialIndexTemplate
}
