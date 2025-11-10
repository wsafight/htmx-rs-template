# HTMX Plugin System

## 整体架构

```
htmx-plugins/
├── Cargo.toml                 # workspace 定义
├── htmx-core/                 # 核心 trait 和工具
│   ├── src/
│   │   ├── plugin.rs          # Plugin trait 定义
│   │   ├── app.rs             # App 构建器
│   │   └── lib.rs
│   └── Cargo.toml
├── htmx-auth/                 # 认证插件
│   ├── src/
│   │   ├── lib.rs
│   │   ├── routes.rs
│   │   ├── models.rs
│   │   └── plugin.rs
│   ├── templates/
│   │   ├── login.html
│   │   └── register.html
│   ├── static/
│   │   └── auth.css
│   ├── migrations/
│   │   └── 001_users.sql
│   └── Cargo.toml
├── htmx-landing/              # 官网插件
└── my-project/                # 实际项目
    ├── Cargo.toml
    └── src/main.rs
```

## 核心设计

### 1. Plugin Trait (htmx-core)

```rust
pub trait HtmxPlugin: Send + Sync + 'static {
    /// 插件名称（用于路由前缀、日志等）
    fn name(&self) -> &str;
    
    /// 路由前缀，默认 /name
    fn mount_path(&self) -> &str { 
        &format!("/{}", self.name()) 
    }
    
    /// 注册路由
    fn routes(&self) -> Router;
    
    /// 数据库迁移
    fn migrations(&self) -> Vec<&'static str> { vec![] }
    
    /// 是否需要认证（可选钩子）
    fn requires_auth(&self) -> bool { false }
    
    /// 初始化钩子（可选，用于依赖注入）
    fn on_init(&mut self, ctx: &PluginContext) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

pub struct PluginContext {
    pub pool: SqlitePool,
    pub config: Arc<AppConfig>,
}
```

### 2. App 构建器

```rust
pub struct HtmxApp {
    plugins: Vec<Box<dyn HtmxPlugin>>,
    pool: Option<SqlitePool>,
    config: AppConfig,
}

impl HtmxApp {
    pub fn new() -> Self { ... }
    
    pub fn plugin<P: HtmxPlugin>(mut self, plugin: P) -> Self {
        self.plugins.push(Box::new(plugin));
        self
    }
    
    pub fn with_db(mut self, pool: SqlitePool) -> Self {
        self.pool = Some(pool);
        self
    }
    
    pub async fn build(mut self) -> Result<Router, Box<dyn Error>> {
        let ctx = PluginContext { 
            pool: self.pool.unwrap(), 
            config: Arc::new(self.config) 
        };
        
        // 运行迁移
        for plugin in &self.plugins {
            for migration in plugin.migrations() {
                run_migration(&ctx.pool, migration).await?;
            }
        }
        
        // 初始化插件
        for plugin in &mut self.plugins {
            plugin.on_init(&ctx)?;
        }
        
        // 组装路由
        let mut app = Router::new();
        for plugin in self.plugins {
            let routes = plugin.routes()
                .layer(Extension(ctx.pool.clone()));
            app = app.nest(plugin.mount_path(), routes);
        }
        
        Ok(app)
    }
}
```

## 插件实现示例

### htmx-auth 插件

```rust
// htmx-auth/src/lib.rs
pub struct AuthPlugin {
    config: AuthConfig,
}

impl AuthPlugin {
    pub fn new() -> Self {
        Self { config: AuthConfig::default() }
    }
    
    pub fn with_session_duration(mut self, duration: Duration) -> Self {
        self.config.session_duration = duration;
        self
    }
}

impl HtmxPlugin for AuthPlugin {
    fn name(&self) -> &str { "auth" }
    
    fn routes(&self) -> Router {
        Router::new()
            .route("/login", get(login_page).post(login_submit))
            .route("/register", get(register_page).post(register_submit))
            .route("/logout", post(logout))
            // 嵌入静态资源
            .route("/static/*path", get(serve_static))
    }
    
    fn migrations(&self) -> Vec<&'static str> {
        vec![
            include_str!("../migrations/001_users.sql"),
            include_str!("../migrations/002_sessions.sql"),
        ]
    }
    
    fn requires_auth(&self) -> bool { false }
}

// 使用 RustEmbed 打包资源
#[derive(RustEmbed)]
#[folder = "static/"]
struct AuthStatic;

#[derive(RustEmbed)]
#[folder = "templates/"]
struct AuthTemplates;
```

## 使用方式

```rust
// my-project/src/main.rs
use htmx_core::HtmxApp;
use htmx_auth::AuthPlugin;
use htmx_landing::LandingPlugin;

#[tokio::main]
async fn main() {
    let pool = create_pool().await?;
    
    let app = HtmxApp::new()
        .plugin(LandingPlugin::new())
        .plugin(AuthPlugin::new()
            .with_session_duration(Duration::from_hours(24))
        )
        .plugin(DashboardPlugin::new())
        .with_db(pool)
        .build()
        .await?;
    
    // 路由自动为:
    // /landing/*
    // /auth/login, /auth/register
    // /dashboard/*
    
    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;
}
```

## 高级特性

### 1. 插件间通信

```rust
pub trait MessageBus {
    fn emit(&self, event: Event);
    fn subscribe(&self, listener: EventListener);
}

// AuthPlugin 发出登录事件
bus.emit(Event::UserLoggedIn { user_id: 123 });

// DashboardPlugin 监听
bus.subscribe(|event| {
    if let Event::UserLoggedIn { user_id } = event {
        // 初始化用户仪表盘
    }
});
```

### 2. 模板继承（类似 Django）

```html
<!-- base.html (由 core 提供) -->
<html>
<head>{% block head %}{% endblock %}</head>
<body>
    <nav>{% block nav %}{% endblock %}</nav>
    {% block content %}{% endblock %}
</body>
</html>

<!-- auth/login.html -->
{% extends "base.html" %}
{% block content %}
<form hx-post="/auth/login">...</form>
{% endblock %}
```

### 3. 配置覆盖

```toml
# my-project/config.toml
[auth]
session_duration = "24h"
allow_registration = true

[landing]
title = "我的产品"
theme = "dark"
```

### 4. 版本兼容

```toml
[dependencies]
htmx-core = "0.1"
htmx-auth = { version = "0.1", features = ["oauth", "2fa"] }
htmx-landing = "0.1"
```

## 优势总结

1. **复用性**: 写一次，到处用
2. **模块化**: 每个插件独立开发、测试
3. **灵活性**: 可选功能、可配置
4. **零拷贝**: `RustEmbed` 编译时打包
5. **类型安全**: Rust 保证接口正确
6. **渐进式**: 可从单体迁移到插件
7. **生态**: 可发布到 crates.io 共享

## 插件开发指南

### 创建新插件

1. 在 `plugins/` 目录下创建新的插件目录
2. 添加 `Cargo.toml` 和标准目录结构
3. 实现 `HtmxPlugin` trait
4. 使用 `RustEmbed` 打包静态资源和模板
5. 提供数据库迁移脚本（如需要）

### 最佳实践

- 插件应该是自包含的，不依赖其他插件
- 使用 feature flags 控制可选功能
- 提供清晰的配置接口
- 编写完整的文档和示例
- 遵循语义化版本控制
