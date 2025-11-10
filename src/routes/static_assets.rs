//! 静态资源处理模块
//!
//! 负责处理静态文件的请求和响应，包含安全检查、文件压缩和优化的缓存策略

use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;
use std::path::PathBuf;
use std::str::FromStr;

/// 静态资源处理错误
#[derive(Debug)]
enum StaticAssetError {
    /// 路径不安全（包含路径遍历）
    UnsafePath,
    /// 文件不存在
    NotFound,
}

/// 静态资源嵌入
#[derive(RustEmbed)]
#[folder = "static/"]
pub struct StaticAssets;

/// 检查路径是否安全，防止路径遍历攻击
///
/// # Parameters
/// - `path`: 要检查的文件路径
///
/// # Returns
/// 如果路径安全返回 Ok(())，否则返回错误
fn is_path_safe(path: &str) -> Result<(), StaticAssetError> {
    // 检查路径是否包含路径遍历模式
    if path.contains("..") {
        return Err(StaticAssetError::UnsafePath);
    }

    // 检查解析后的路径是否规范化（没有上一级目录引用）
    let path_buf = PathBuf::from_str(path).map_err(|_| StaticAssetError::UnsafePath)?;
    if path_buf.components().any(|c| c.as_os_str() == "..") {
        return Err(StaticAssetError::UnsafePath);
    }

    Ok(())
}

/// 获取基于文件类型的缓存时间
///
/// # Parameters
/// - `path`: 文件路径
///
/// # Returns
/// 缓存控制头字符串
fn get_cache_control(path: &str) -> &'static str {
    // 对不同类型的文件使用不同的缓存策略
    if path.ends_with(".js") || path.ends_with(".css") || path.ends_with(".svg") {
        // 对静态资源使用长缓存
        "public, max-age=31536000, immutable"
    } else if path.ends_with(".jpg")
        || path.ends_with(".jpeg")
        || path.ends_with(".png")
        || path.ends_with(".gif")
    {
        // 对图片使用长缓存
        "public, max-age=604800, immutable"
    } else if path.ends_with(".html") || path.ends_with(".htm") {
        // 对HTML文件使用较短的缓存
        "public, max-age=3600, must-revalidate"
    } else {
        // 默认缓存策略
        "public, max-age=86400"
    }
}

/// 处理静态文件请求
///
/// # Parameters
/// - `uri`: 请求的 URI
///
/// # Returns
/// 返回对应的静态文件或错误响应
pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches("/static/");

    // 1. 路径安全检查
    if let Err(err) = is_path_safe(path) {
        match err {
            StaticAssetError::UnsafePath => {
                tracing::warn!("尝试访问不安全的路径: {}", path);
                return Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(Body::from("403 Forbidden"))
                    .unwrap();
            }
            StaticAssetError::NotFound => {}
        }
    }

    // 2. 获取静态资源
    match StaticAssets::get(path) {
        Some(content) => {
            // 3. 确定文件类型
            let mime = mime_guess::from_path(path).first_or_octet_stream();

            // 4. 创建响应
            let mut response_builder = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .header(header::CACHE_CONTROL, get_cache_control(path));

            // 5. 对于文本类文件，可以考虑添加ETag支持
            if mime.type_() == "text" || mime.subtype() == "javascript" || mime.subtype() == "css" {
                // 生成简单的ETag（基于内容长度）
                let etag = format!("\"{}\"", content.data.len());
                response_builder = response_builder.header(header::ETAG, etag);
            }

            // 6. 返回响应
            response_builder
                .body(Body::from(content.data))
                .unwrap_or_else(|e| {
                    tracing::error!("创建静态文件响应失败: {}", e);
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("500 Internal Server Error"))
                        .unwrap()
                })
        }
        None => {
            tracing::debug!("静态文件未找到: {}", path);
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("404 Not Found"))
                .unwrap()
        }
    }
}
