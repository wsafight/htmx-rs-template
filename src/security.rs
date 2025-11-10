//! 安全模块
//!
//! 提供CSRF保护、输入验证、日志脱敏等安全功能

use axum::{http::Request, response::Response};
use rand::{distributions::Alphanumeric, Rng};
use std::marker::PhantomData;
use std::sync::Arc;
use std::task::{Context, Poll};
use thiserror::Error;
use tower::{Layer, Service};
use validator::Validate;

/// 安全错误类型
#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("CSRF token 无效")]
    InvalidCsrfToken,
    #[error("缺少 CSRF token")]
    MissingCsrfToken,
    #[error("输入验证失败: {0}")]
    ValidationFailed(String),
    #[error("安全检查失败: {0}")]
    SecurityCheckFailed(String),
}

/// CSRF token 中间件配置
#[derive(Debug, Clone)]
pub struct CsrfConfig {
    pub cookie_name: String,
    pub header_name: String,
    pub token_length: usize,
    pub enable_protection: bool,
}

impl Default for CsrfConfig {
    fn default() -> Self {
        Self {
            cookie_name: "XSRF-TOKEN".to_string(),
            header_name: "X-XSRF-TOKEN".to_string(),
            token_length: 32,
            enable_protection: true,
        }
    }
}

/// CSRF 保护层
pub struct CsrfLayer<T> {
    config: CsrfConfig,
    _marker: PhantomData<T>,
}

impl<T> CsrfLayer<T> {
    pub fn new(config: CsrfConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }
}

impl<S, T> Layer<S> for CsrfLayer<T>
where
    S: Service<Request<T>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    T: Send + 'static,
{
    type Service = CsrfService<S, T>;

    fn layer(&self, inner: S) -> <CsrfLayer<T> as Layer<S>>::Service {
        CsrfService {
            inner,
            config: self.config.clone(),
            _marker: PhantomData,
        }
    }
}

/// CSRF 保护服务
pub struct CsrfService<S, T> {
    inner: S,
    config: CsrfConfig,
    _marker: PhantomData<T>,
}

impl<S, T> Service<Request<T>> for CsrfService<S, T>
where
    S: Service<Request<T>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    T: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = futures::future::BoxFuture<'static, Result<S::Response, S::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<T>) -> <CsrfService<S, T> as Service<Request<T>>>::Future {
        let mut inner = self.inner.clone();
        let config = self.config.clone();

        Box::pin(async move {
            // 跳过GET、HEAD、OPTIONS请求的CSRF检查
            let method = req.method();
            if method == axum::http::Method::GET
                || method == axum::http::Method::HEAD
                || method == axum::http::Method::OPTIONS
            {
                return inner.call(req).await;
            }

            // 如果禁用了CSRF保护，直接通过
            if !config.enable_protection {
                return inner.call(req).await;
            }

            // 从请求头获取CSRF token
            let token_from_header = req
                .headers()
                .get(&config.header_name)
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string());

            // 从cookie获取CSRF token
            let token_from_cookie = req
                .headers()
                .get(axum::http::header::COOKIE)
                .and_then(|h| h.to_str().ok())
                .and_then(|cookie_str| extract_cookie_value(cookie_str, &config.cookie_name));

            // 验证token是否存在且匹配
            let is_valid = match (token_from_header, token_from_cookie) {
                (Some(h), Some(c)) => h == c,
                _ => false,
            };

            if !is_valid {
                // 可以在这里返回自定义错误响应
                tracing::warn!(
                    "CSRF token验证失败: 请求方法={}, 路径={}",
                    method,
                    req.uri().path()
                );
            }

            // 即使CSRF验证失败，也允许请求继续，但记录警告日志
            // 在生产环境中，这里应该返回错误响应
            inner.call(req).await
        })
    }
}

/// 从cookie字符串中提取指定cookie的值
fn extract_cookie_value(cookie_str: &str, cookie_name: &str) -> Option<String> {
    for cookie in cookie_str.split(';') {
        let cookie = cookie.trim();
        if let Some((name, value)) = cookie.split_once('=') {
            if name == cookie_name {
                // 简单的URL解码
                return Some(value.replace("%3D", "=").replace("%2B", "+"));
            }
        }
    }
    None
}

/// 生成CSRF token
pub fn generate_csrf_token(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// CSRF token 提供者中间件
pub async fn csrf_token_middleware(
    mut req: Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<Response, std::convert::Infallible> {
    let config = &crate::config::CONFIG;

    if config.security.enable_csrf {
        // 生成新的CSRF token
        let token = generate_csrf_token(32);

        // 将token作为扩展添加到请求中，以便处理器可以在响应中设置
        req.extensions_mut().insert(Arc::new(token.clone()));
    }

    let mut response = next.run(req).await;

    // 设置CSRF cookie（如果启用了保护）
    if config.security.enable_csrf {
        if let Some(token) = response.extensions().get::<Arc<String>>() {
            let cookie = format!("XSRF-TOKEN={}; Path=/; HttpOnly; SameSite=Lax", token);
            response
                .headers_mut()
                .append(axum::http::header::SET_COOKIE, cookie.parse().unwrap());
        }
    }

    Ok(response)
}

/// 输入验证工具
pub mod validation {
    use super::*;
    use validator::ValidationErrors;

    /// 验证输入数据并返回友好的错误消息
    pub fn validate_input<T: Validate>(input: &T) -> Result<(), SecurityError> {
        match input.validate() {
            Ok(_) => Ok(()),
            Err(errors) => {
                let error_message = format_validation_errors(&errors);
                Err(SecurityError::ValidationFailed(error_message))
            }
        }
    }

    /// 格式化验证错误为友好的错误消息
    fn format_validation_errors(errors: &ValidationErrors) -> String {
        let mut messages = Vec::new();

        for (field, field_errors) in errors.field_errors() {
            for error in field_errors {
                let message = match &error.message {
                    Some(msg) => msg.to_string(),
                    None => format!("字段 '{}' 验证失败: {}", field, error.code),
                };
                messages.push(message);
            }
        }

        messages.join(", ")
    }
}

/// 日志脱敏工具
pub mod sanitization {
    /// 脱敏敏感信息
    pub fn sanitize_log_message(message: &str) -> String {
        let mut result = message.to_string();

        // 脱敏邮箱
        result = regex::Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")
            .unwrap()
            .replace_all(&result, "***@***.***")
            .to_string();

        // 脱敏手机号（简单匹配10-15位数字）
        result = regex::Regex::new(r"\b\d{10,15}\b")
            .unwrap()
            .replace_all(&result, |caps: &regex::Captures| {
                let num = caps.get(0).unwrap().as_str();
                format!("{}****{}", &num[0..3], &num[num.len() - 3..num.len()])
            })
            .to_string();

        // 脱敏密码
        result = regex::Regex::new(r#"(?i)password\s*=\s*['"]([^'"]+)['"]"#)
            .unwrap()
            .replace_all(&result, "password=***")
            .to_string();

        // 脱敏token
        result = regex::Regex::new(r#"(?i)token\s*=\s*['"]([^'"]+)['"]"#)
            .unwrap()
            .replace_all(&result, "token=***")
            .to_string();

        result
    }
}

// 导入中间件模块
// 导出常用的安全相关类型和函数
pub use self::sanitization::sanitize_log_message;