mod models;
mod routes;
mod static_handler;

use askama::Template;
use axum::Router;
use htmx_core::HtmxPlugin;
use serde::{Deserialize, Serialize};

pub use routes::create_routes;

/// 官网插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandingConfig {
    pub title: String,
    pub subtitle: String,
    pub features: Vec<Feature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub icon: String,
    pub title: String,
    pub description: String,
}

impl Default for LandingConfig {
    fn default() -> Self {
        Self {
            title: "HTMX Rust 模板".to_string(),
            subtitle: "快速构建现代化的 Web 应用".to_string(),
            features: vec![
                Feature {
                    icon: "🚀".to_string(),
                    title: "快速开发".to_string(),
                    description: "使用 HTMX 和 Rust 快速构建交互式应用".to_string(),
                },
                Feature {
                    icon: "⚡".to_string(),
                    title: "高性能".to_string(),
                    description: "基于 Axum 和 Tokio，提供卓越的性能".to_string(),
                },
                Feature {
                    icon: "🔒".to_string(),
                    title: "类型安全".to_string(),
                    description: "Rust 的类型系统确保代码的安全性".to_string(),
                },
            ],
        }
    }
}

/// 官网插件
pub struct LandingPlugin {
    config: LandingConfig,
}

impl LandingPlugin {
    pub fn new() -> Self {
        Self {
            config: LandingConfig::default(),
        }
    }

    pub fn with_config(mut self, config: LandingConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.config.title = title.into();
        self
    }

    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.config.subtitle = subtitle.into();
        self
    }
}

impl Default for LandingPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl HtmxPlugin for LandingPlugin {
    fn name(&self) -> &str {
        "landing"
    }

    fn routes(&self) -> Router {
        create_routes(self.config.clone())
    }

    fn requires_auth(&self) -> bool {
        false
    }
}
