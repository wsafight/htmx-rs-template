//! 应用配置管理模块
//!
//! 统一管理应用的所有配置，支持从环境变量和配置文件加载配置

use figment::{
    providers::{Env, Format, Toml},
    Error as FigmentError, Figment,
};
use serde::Deserialize;
use std::path::PathBuf;
use thiserror::Error;

/// 配置加载错误类型
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("配置加载错误: {0}")]
    Loading(#[from] FigmentError),
    #[error("配置验证错误: {0}")]
    Validation(String),
}

/// 数据库配置
#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    #[allow(dead_code)]
    pub url: Option<String>,
    pub max_connections: u32,
    pub min_connections: u32,
    #[allow(dead_code)]
    pub acquire_timeout_seconds: u64,
    #[allow(dead_code)]
    pub idle_timeout_seconds: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: None,
            max_connections: 10,
            min_connections: 2,
            acquire_timeout_seconds: 5,
            idle_timeout_seconds: 300,
        }
    }
}

/// 服务器配置
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    #[allow(dead_code)]
    pub worker_threads: Option<usize>,
    pub graceful_shutdown_timeout_seconds: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            worker_threads: None,
            graceful_shutdown_timeout_seconds: 5,
        }
    }
}

impl ServerConfig {
    /// 获取服务器地址
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// 安全配置
#[derive(Debug, Deserialize, Clone)]
pub struct SecurityConfig {
    pub cors_allow_origins: Vec<String>,
    #[allow(dead_code)]
    pub rate_limit_per_minute: u64,
    #[allow(dead_code)]
    pub enable_csrf: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            cors_allow_origins: vec![
                "http://localhost:3000".to_string(),
                "http://127.0.0.1:3000".to_string(),
            ],
            rate_limit_per_minute: 60,
            enable_csrf: true,
        }
    }
}

/// 应用配置
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub security: SecurityConfig,
    pub log_level: String,
    pub environment: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            server: ServerConfig::default(),
            security: SecurityConfig::default(),
            log_level: "info".to_string(),
            environment: "development".to_string(),
        }
    }
}

impl AppConfig {
    /// 从默认位置加载配置
    pub fn load() -> Result<Self, ConfigError> {
        // 配置文件搜索路径
        let config_paths = [
            PathBuf::from("./config.toml"),
            PathBuf::from("../config.toml"),
            PathBuf::from("./config/config.toml"),
        ];

        // 创建配置构建器
        let mut figment = Figment::new();

        // 加载存在的配置文件
        for path in config_paths {
            if path.exists() {
                tracing::info!("从配置文件加载: {}", path.display());
                figment = figment.merge(Toml::file(path));
                break; // 只加载第一个存在的配置文件
            }
        }

        // 从环境变量加载（优先级最高）
        figment = figment.merge(Env::prefixed("APP_").split("."));

        // 构建配置
        let config: AppConfig = figment.extract()?;

        // 验证配置
        config.validate()?;

        Ok(config)
    }

    /// 验证配置
    fn validate(&self) -> Result<(), ConfigError> {
        // 环境必须是 development、staging 或 production
        if !matches!(
            self.environment.to_lowercase().as_str(),
            "development" | "staging" | "production"
        ) {
            return Err(ConfigError::Validation(
                "环境必须是 development、staging 或 production".to_string(),
            ));
        }

        // 验证日志级别
        if !matches!(
            self.log_level.to_lowercase().as_str(),
            "error" | "warn" | "info" | "debug" | "trace"
        ) {
            return Err(ConfigError::Validation(
                "日志级别必须是 error、warn、info、debug 或 trace".to_string(),
            ));
        }

        // 验证数据库配置
        if self.database.max_connections < self.database.min_connections {
            return Err(ConfigError::Validation(
                "最大连接数不能小于最小连接数".to_string(),
            ));
        }

        Ok(())
    }

    /// 是否为生产环境
    #[allow(dead_code)]
    pub fn is_production(&self) -> bool {
        self.environment.to_lowercase() == "production"
    }

    /// 是否为开发环境
    #[allow(dead_code)]
    pub fn is_development(&self) -> bool {
        self.environment.to_lowercase() == "development"
    }
}

// 提供一个全局配置实例的访问方式
lazy_static::lazy_static! {
    pub static ref CONFIG: AppConfig = AppConfig::load()
        .unwrap_or_else(|e| {
            eprintln!("警告: 无法加载配置: {}. 使用默认配置.", e);
            AppConfig::default()
        });
}
