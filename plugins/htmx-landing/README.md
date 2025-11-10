# HTMX Landing Plugin

官网/落地页插件，提供开箱即用的产品介绍页面。

## 特性

- 🎨 现代化响应式设计
- ⚡ HTMX 动态加载统计数据
- 📦 嵌入式静态资源（CSS）
- 🔧 可配置的内容和特性展示
- 🎯 零依赖的前端（仅需 HTMX CDN）

## 安装

```toml
[dependencies]
htmx-landing = { path = "../plugins/htmx-landing" }
```

## 使用

### 基础使用

```rust
use htmx_core::HtmxApp;
use htmx_landing::LandingPlugin;

let app = HtmxApp::new()
    .plugin(LandingPlugin::new())
    .build()
    .await?;
```

访问 `http://localhost:3000/landing/` 查看官网页面。

### 自定义配置

```rust
use htmx_landing::{LandingPlugin, LandingConfig, Feature};

let config = LandingConfig {
    title: "我的产品".to_string(),
    subtitle: "最好的解决方案".to_string(),
    features: vec![
        Feature {
            icon: "🎯".to_string(),
            title: "精准定位".to_string(),
            description: "为您量身定制的解决方案".to_string(),
        },
        Feature {
            icon: "💡".to_string(),
            title: "创新技术".to_string(),
            description: "采用最新的技术栈".to_string(),
        },
    ],
};

let app = HtmxApp::new()
    .plugin(LandingPlugin::new().with_config(config))
    .build()
    .await?;
```

### 链式配置

```rust
let app = HtmxApp::new()
    .plugin(
        LandingPlugin::new()
            .with_title("我的产品")
            .with_subtitle("让工作更简单")
    )
    .build()
    .await?;
```

## 路由

- `GET /landing/` - 首页
- `GET /landing/stats` - 统计数据（HTMX 动态加载）
- `GET /landing/static/*` - 静态资源

## 自定义

### 修改模板

编辑 `templates/index.html` 和 `templates/stats.html`。

### 修改样式

编辑 `static/style.css`。

### 添加统计数据

修改 `src/routes.rs` 中的 `stats()` 函数，从数据库获取真实数据：

```rust
async fn stats(
    Extension(pool): Extension<SqlitePool>
) -> impl IntoResponse {
    let user_count = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap_or(0);
    
    StatsTemplate {
        user_count,
        project_count: 500,
        satisfaction: 98,
    }
}
```

## 示例

完整示例请参考 `examples/` 目录。

## License

MIT
