//! 静态资源处理模块
//! 
//! 负责处理静态文件的请求和响应

use axum::{body::Body, http::{header, StatusCode, Uri}, response::{IntoResponse, Response}};
use rust_embed::RustEmbed;

/// 静态资源嵌入
#[derive(RustEmbed)]
#[folder = "static/"]
pub struct StaticAssets;

/// 处理静态文件请求
/// 
/// # Parameters
/// - `uri`: 请求的 URI
/// 
/// # Returns
/// 返回对应的静态文件或 404 响应
pub async fn static_handler(uri: Uri) -> impl IntoResponse {
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