use crate::plugin::{HtmxPlugin, PluginContext};
use axum::{Extension, Router};
use sqlx::SqlitePool;
use std::error::Error;
use std::sync::Arc;

/// HTMX 应用构建器
///
/// 用于组装插件和配置应用
pub struct HtmxApp {
    plugins: Vec<Box<dyn HtmxPlugin>>,
    pool: Option<SqlitePool>,
    config: serde_json::Value,
}

impl HtmxApp {
    /// 创建新的应用构建器
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            pool: None,
            config: serde_json::json!({}),
        }
    }

    /// 注册插件
    pub fn plugin<P: HtmxPlugin>(mut self, plugin: P) -> Self {
        self.plugins.push(Box::new(plugin));
        self
    }

    /// 设置数据库连接池
    pub fn with_db(mut self, pool: SqlitePool) -> Self {
        self.pool = Some(pool);
        self
    }

    /// 设置配置
    pub fn with_config(mut self, config: serde_json::Value) -> Self {
        self.config = config;
        self
    }

    /// 构建应用
    ///
    /// 执行迁移、初始化插件、组装路由
    pub async fn build(mut self) -> Result<Router, Box<dyn Error>> {
        let pool = self.pool.ok_or("Database pool is required")?;

        let ctx = PluginContext {
            pool: pool.clone(),
            config: Arc::new(self.config),
        };

        // 运行数据库迁移
        for plugin in &self.plugins {
            tracing::info!("Running migrations for plugin: {}", plugin.name());
            for (idx, migration) in plugin.migrations().iter().enumerate() {
                tracing::debug!("Running migration {} for {}", idx + 1, plugin.name());
                sqlx::query(migration)
                    .execute(&ctx.pool)
                    .await
                    .map_err(|e| format!("Migration failed for {}: {}", plugin.name(), e))?;
            }
        }

        // 初始化插件
        for plugin in &mut self.plugins {
            tracing::info!("Initializing plugin: {}", plugin.name());
            plugin.on_init(&ctx)?;
        }

        // 组装路由
        let mut app = Router::new();

        for plugin in self.plugins {
            let mount_path = plugin.mount_path();
            tracing::info!(
                "Mounting plugin '{}' at path: {}",
                plugin.name(),
                mount_path
            );

            let routes = plugin.routes().layer(Extension(ctx.pool.clone()));

            app = app.nest(&mount_path, routes);
        }

        Ok(app)
    }
}

impl Default for HtmxApp {
    fn default() -> Self {
        Self::new()
    }
}
