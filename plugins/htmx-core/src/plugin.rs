use axum::Router;
use sqlx::SqlitePool;
use std::error::Error;
use std::sync::Arc;

/// 插件上下文，包含共享资源
pub struct PluginContext {
    pub pool: SqlitePool,
    pub config: Arc<serde_json::Value>,
}

/// HTMX 插件 trait
///
/// 实现此 trait 以创建可复用的 HTMX 模块
pub trait HtmxPlugin: Send + Sync + 'static {
    /// 插件名称（用于路由前缀、日志等）
    fn name(&self) -> &str;

    /// 路由挂载路径，默认为 /name
    fn mount_path(&self) -> String {
        format!("/{}", self.name())
    }

    /// 注册路由
    ///
    /// 返回包含所有路由的 Router
    fn routes(&self) -> Router;

    /// 数据库迁移 SQL
    ///
    /// 返回迁移 SQL 字符串数组，按顺序执行
    fn migrations(&self) -> Vec<&'static str> {
        vec![]
    }

    /// 是否需要认证
    ///
    /// 如果返回 true，所有路由将应用认证中间件
    fn requires_auth(&self) -> bool {
        false
    }

    /// 初始化钩子
    ///
    /// 在插件注册时调用，可用于依赖注入、资源初始化等
    fn on_init(&mut self, _ctx: &PluginContext) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    /// 关闭钩子
    ///
    /// 在应用关闭时调用，可用于清理资源
    fn on_shutdown(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
