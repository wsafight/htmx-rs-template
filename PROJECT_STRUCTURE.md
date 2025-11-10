# 项目结构说明

## 📁 目录结构

```
htmx-rs-template/
├── src/                      # Rust 源代码
│   ├── main.rs              # 应用入口点，配置服务器和路由
│   └── routes/              # 路由处理模块
│       ├── mod.rs           # 路由模块入口
│       ├── todos.rs         # 待办事项 CRUD 操作
│       ├── users.rs         # 用户列表和搜索
│       └── modal.rs         # 模态框示例
│
├── templates/               # HTML 模板 (Askama)
│   ├── base.html           # 基础布局（导航栏、页脚）
│   ├── index.html          # 首页
│   ├── todos/              # 待办事项模板
│   │   ├── list.html       # 待办列表页面
│   │   ├── item.html       # 单个待办项
│   │   └── create_form.html # 创建表单
│   ├── users/              # 用户相关模板
│   │   ├── list.html       # 用户列表页面
│   │   └── search_results.html # 搜索结果
│   └── modal/              # 模态框模板
│       └── example.html    # 模态框内容
│
├── static/                  # 静态资源
│   └── css/
│       └── style.css       # 全局样式表
│
├── Cargo.toml              # Rust 项目配置和依赖
├── Cargo.lock              # 依赖锁定文件
├── .gitignore              # Git 忽略文件
├── README.md               # 项目文档
├── QUICKSTART.md           # 快速入门指南
└── PROJECT_STRUCTURE.md    # 本文件
```

## 🔍 核心文件说明

### `src/main.rs`
应用的入口点，负责：
- 初始化日志系统
- 配置路由
- 启动 HTTP 服务器

```rust
let app = Router::new()
    .route("/", get(routes::index))          // 首页
    .route("/todos", get(routes::todos::list)) // 待办列表
    .nest_service("/static", ServeDir::new("static")) // 静态文件
    .layer(TraceLayer::new_for_http());      // 日志中间件
```

### `src/routes/todos.rs`
待办事项的完整 CRUD 实现：
- **list()**: 显示所有待办事项
- **create_form()**: 返回创建表单
- **create()**: 处理表单提交，创建新任务
- **delete()**: 删除指定任务
- **toggle()**: 切换任务完成状态

数据存储在内存中（使用 `lazy_static`），适合演示和开发。

### `src/routes/users.rs`
用户列表和搜索功能：
- **list()**: 显示所有用户
- **search()**: 实时搜索用户（支持防抖）

### `templates/base.html`
基础模板，所有页面都继承自它：
- 导航栏
- HTMX CDN 引入
- CSS 样式引入
- 页脚

```html
{% extends "base.html" %}
{% block content %}
  <!-- 页面内容 -->
{% endblock %}
```

### `static/css/style.css`
统一的样式文件，包含：
- CSS 变量定义（颜色、间距等）
- 响应式布局
- 组件样式（按钮、卡片、表单等）
- HTMX 动画效果

## 🔄 数据流

### 典型的 HTMX 请求流程

```
1. 用户交互
   ↓
2. HTMX 发送 HTTP 请求
   ↓
3. Axum 路由匹配
   ↓
4. 路由处理函数执行
   ↓
5. Askama 渲染模板
   ↓
6. 返回 HTML 片段
   ↓
7. HTMX 更新 DOM
```

### 示例：删除待办事项

```html
<!-- 1. 用户点击删除按钮 -->
<button hx-delete="/todos/1" hx-target="#todo-1" hx-swap="outerHTML">
  删除
</button>

<!-- 2. HTMX 发送 DELETE /todos/1 -->

<!-- 3. Rust 处理函数 -->
pub async fn delete(Path(id): Path<usize>) -> impl IntoResponse {
    let mut todos = TODOS.lock().unwrap();
    todos.retain(|t| t.id != id);
    StatusCode::OK  // 返回 200，HTMX 删除元素
}

<!-- 4. HTMX 删除 #todo-1 元素 -->
```

## 🎨 模板继承

```
base.html (基础布局)
    ├── index.html (首页)
    ├── todos/list.html (待办列表)
    │       └── includes todos/item.html
    └── users/list.html (用户列表)
            └── dynamically loads users/search_results.html
```

## 📊 技术栈详解

### 后端 (Rust)
- **Axum**: 基于 Tokio 的 Web 框架，性能优秀
- **Tokio**: 异步运行时
- **Tower**: 中间件系统
- **Serde**: JSON 序列化/反序列化

### 模板引擎 (Askama)
- 类似 Jinja2 的模板语法
- **编译时检查**：模板错误在编译时发现
- 类型安全：变量类型在编译时验证
- 高性能：模板编译为 Rust 代码

### 前端 (HTMX)
- 通过 HTML 属性驱动交互
- 无需编写 JavaScript
- 支持所有 HTTP 方法
- 自动处理请求和 DOM 更新

## 🔌 扩展点

### 添加数据库
推荐使用 SQLx：
```rust
// Cargo.toml
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-native-tls"] }

// 使用
let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;
```

### 添加认证
推荐使用 axum-login 或 tower-sessions：
```rust
// 在路由中添加中间件
.layer(AuthLayer::new(session_store))
```

### 添加 WebSocket
Axum 原生支持 WebSocket：
```rust
use axum::extract::ws::WebSocket;

async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}
```

## 📝 命名约定

- **路由函数**: 动词命名 (`list`, `create`, `update`, `delete`)
- **模板结构体**: `*Template` 后缀 (`IndexTemplate`, `TodoListTemplate`)
- **CSS 类**: kebab-case (`user-card`, `todo-item`)
- **Rust 类型**: PascalCase (`User`, `Todo`)

## 🚀 性能优化建议

1. **静态资源**：考虑使用 CDN
2. **数据库连接池**：使用 SQLx 连接池
3. **缓存**：添加 Redis 缓存层
4. **压缩**：使用 tower-http 的压缩中间件
5. **编译优化**：release 模式编译

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

## 📖 相关文档

- [Axum 文档](https://docs.rs/axum/latest/axum/)
- [Askama 文档](https://docs.rs/askama/latest/askama/)
- [HTMX 文档](https://htmx.org/docs/)
- [Tokio 教程](https://tokio.rs/tokio/tutorial)
