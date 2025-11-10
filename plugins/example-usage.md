# 插件系统使用示例

## 项目结构

```
my-project/
├── Cargo.toml
└── src/
    └── main.rs
```

## Cargo.toml

```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2021"

[dependencies]
# 核心框架
htmx-core = { path = "../plugins/htmx-core" }

# 插件
htmx-landing = { path = "../plugins/htmx-landing" }
# htmx-auth = { path = "../plugins/htmx-auth" }  # 待实现

# 基础依赖
axum = "0.7"
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.8", features = ["sqlite"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

## src/main.rs

```rust
use htmx_core::HtmxApp;
use htmx_landing::LandingPlugin;
use sqlx::sqlite::SqlitePoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 创建数据库连接池
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:app.db")
        .await?;

    // 构建应用
    let app = HtmxApp::new()
        .plugin(
            LandingPlugin::new()
                .with_title("我的产品")
                .with_subtitle("快速构建现代化应用")
        )
        .with_db(pool)
        .build()
        .await?;

    // 启动服务器
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("🚀 服务器启动: http://localhost:3000");
    tracing::info!("📱 访问官网: http://localhost:3000/landing/");
    
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
```

## 运行

```bash
cargo run
```

访问 http://localhost:3000/landing/ 查看官网页面。

## 添加更多插件

```rust
let app = HtmxApp::new()
    // 官网插件
    .plugin(
        LandingPlugin::new()
            .with_title("我的产品")
    )
    // 认证插件（待实现）
    // .plugin(AuthPlugin::new())
    // 仪表盘插件（待实现）
    // .plugin(DashboardPlugin::new())
    .with_db(pool)
    .build()
    .await?;
```

每个插件会自动挂载到对应的路径：
- `/landing/*` - 官网
- `/auth/*` - 认证
- `/dashboard/*` - 仪表盘

## 自定义配置

```rust
use htmx_landing::{LandingConfig, Feature};

let landing_config = LandingConfig {
    title: "我的产品".to_string(),
    subtitle: "最好的解决方案".to_string(),
    features: vec![
        Feature {
            icon: "🎯".to_string(),
            title: "精准定位".to_string(),
            description: "为您量身定制".to_string(),
        },
        Feature {
            icon: "💡".to_string(),
            title: "创新技术".to_string(),
            description: "最新技术栈".to_string(),
        },
        Feature {
            icon: "🔒".to_string(),
            title: "安全可靠".to_string(),
            description: "企业级安全".to_string(),
        },
    ],
};

let app = HtmxApp::new()
    .plugin(LandingPlugin::new().with_config(landing_config))
    .with_db(pool)
    .build()
    .await?;
```

## 下一步

1. 实现更多插件（认证、博客、仪表盘等）
2. 添加插件间通信机制
3. 支持插件配置文件
4. 发布到 crates.io
