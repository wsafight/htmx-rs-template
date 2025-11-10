# 项目结构说明

## 📁 目录结构

```
htmx-rs-template/
├── src/                      # Rust 源代码
│   ├── main.rs              # 应用入口点，配置服务器和路由
│   ├── db.rs                # 数据库模块（连接池、初始化、数据填充）
│   └── routes/              # 路由处理模块
│       ├── mod.rs           # 路由模块入口，页面模板定义
│       ├── todos.rs         # 待办事项 CRUD 操作 + 统计
│       ├── users.rs         # 用户列表、搜索、详情
│       └── modal.rs         # 模态框示例
│
├── templates/               # HTML 模板 (Askama)
│   ├── base.html           # 基础布局（导航栏、页脚、CSS/JS 引入）
│   ├── index.html          # 首页完整模板（继承 base.html）
│   ├── todos_full.html     # Todos 完整页面（用于直接访问 /todos）
│   ├── users_full.html     # Users 完整页面（用于直接访问 /users）
│   │
│   ├── pages/              # SPA 页面内容片段（不包含 base.html）
│   │   ├── home.html       # 首页内容片段
│   │   ├── todos.html      # 待办列表页面片段
│   │   └── users.html      # 用户列表页面片段
│   │
│   ├── todos/              # 待办事项组件模板
│   │   ├── item.html       # 单个待办项
│   │   ├── create_form.html # 创建表单
│   │   └── stats.html      # 统计卡片（总数、已完成、待完成）
│   │
│   ├── users/              # 用户相关模板
│   │   ├── search_results.html # 搜索结果列表
│   │   └── detail.html     # 用户详情卡片
│   │
│   └── modal/              # 模态框模板
│       └── example.html    # 模态框内容示例
│
├── static/                  # 静态资源（编译时嵌入到二进制文件）
│   └── css/
│       └── style.css       # 全局样式表
│
├── Cargo.toml              # Rust 项目配置和依赖
├── Cargo.lock              # 依赖锁定文件
├── askama.toml             # Askama 模板引擎配置
├── build.sh                # 优化构建脚本
├── Dockerfile              # Docker 镜像构建文件
├── docker-compose.yml      # Docker Compose 配置
├── .dockerignore           # Docker 忽略文件
├── .gitignore              # Git 忽略文件
├── README.md               # 项目文档
├── QUICKSTART.md           # 快速入门指南
├── BOOTSTRAP_UNOCSS_GUIDE.md # Bootstrap + UnoCSS 集成指南
├── CHEATSHEET.md           # HTMX 速查表
└── PROJECT_STRUCTURE.md    # 本文件
```

## 🔍 核心文件说明

### `src/main.rs`

应用的入口点，负责：
- 初始化日志系统（tracing）
- 创建数据库连接池
- 初始化数据库表结构
- 插入示例数据
- 配置路由和中间件
- 启动 HTTP 服务器
- 处理静态资源（使用 rust-embed）

**路由架构**:

```rust
let app = Router::new()
    // === 完整页面路由（首次访问/直接访问）===
    .route("/", get(routes::index))              // 首页
    .route("/todos", get(routes::todos_page))    // Todos 完整页面
    .route("/users", get(routes::users_page))    // Users 完整页面
    
    // === SPA 页面内容路由（返回 HTML 片段）===
    .route("/page/home", get(routes::page_home))
    .route("/page/todos", get(routes::page_todos))
    .route("/page/users", get(routes::page_users))
    
    // === 待办事项 API ===
    .route("/todos/create", get(routes::todos::create_form))
    .route("/api/todos", post(routes::todos::create))
    .route("/todos/:id", delete(routes::todos::delete))
    .route("/todos/:id/toggle", put(routes::todos::toggle))
    
    // === 用户 API ===
    .route("/users/search", get(routes::users::search))
    .route("/users/:id/detail", get(routes::users::detail))
    
    // === 模态框 ===
    .route("/modal/example", get(routes::modal::example))
    
    // === 静态文件（嵌入式）===
    .route("/static/*path", get(static_handler))
    
    .layer(TraceLayer::new_for_http())  // HTTP 请求日志
    .layer(Extension(pool));             // 数据库连接池注入
```

**关键特性**:
- 使用 `rust-embed` 将 static 目录嵌入到二进制文件
- 静态资源带缓存控制头（max-age=31536000）
- 自动根据文件扩展名设置 MIME 类型

### `src/db.rs`

数据库模块，提供：

1. **连接池创建** (`create_pool`)
   - 默认在可执行文件目录创建 `app.db`
   - 支持 `DATABASE_URL` 环境变量自定义路径
   - 配置最大 5 个连接，3 秒超时

2. **数据库初始化** (`init_db`)
   - 创建 `todos` 表（id, title, completed, created_at）
   - 创建 `users` 表（id, name, email, created_at）

3. **示例数据填充** (`seed_data`)
   - 检查表是否为空，仅在首次启动时插入数据
   - Todos: 3 个示例待办事项
   - Users: 4 个示例用户

**数据表结构**:

```sql
-- 待办事项表
CREATE TABLE todos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 用户表
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### `src/routes/mod.rs`

路由模块入口，定义所有页面模板和路由处理函数。

**模板类型**:

```rust
// 完整页面模板（包含 base.html）
IndexTemplate           // 首页
TodosFullPageTemplate   // /todos 直接访问
UsersFullPageTemplate   // /users 直接访问

// SPA 页面内容片段（仅内容部分）
HomePageTemplate        // /page/home
TodosPageTemplate       // /page/todos
UsersPageTemplate       // /page/users
```

**路由函数职责**:
- `index()`: 返回首页完整模板
- `todos_page()`: 从数据库加载数据，返回 Todos 完整页面
- `users_page()`: 从数据库加载数据，返回 Users 完整页面
- `page_*()`: 返回对应的页面内容片段（用于 SPA 导航）

### `src/routes/todos.rs`

待办事项的完整 CRUD 实现：

**数据模型**:

```rust
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub completed: bool,
}
```

**主要函数**:

- **`get_todos(pool)`**: 从数据库获取所有待办事项（按 ID 降序）
- **`get_stats(pool)`**: 计算统计信息（总数、已完成、待完成）
- **`create_form()`**: 返回创建表单 HTML
- **`create(Form)`**: 
  - 插入新待办到数据库
  - 返回新建的待办项 HTML
  - 使用 OOB Swap 同时更新统计卡片
- **`delete(Path(id))`**: 
  - 从数据库删除指定待办
  - 返回空内容（HTMX 删除元素）
  - 使用 OOB Swap 更新统计卡片
- **`toggle(Path(id))`**: 
  - 切换待办的完成状态
  - 返回更新后的待办项 HTML
  - 使用 OOB Swap 更新统计卡片

**关键技术 - OOB Swap**:

```rust
// 返回主要内容 + OOB 更新统计区域
format!(
    "{}<div id=\"todo-stats\" hx-swap-oob=\"true\">{}</div>",
    todo_html, stats_html
)
```

这样一次响应可以更新多个页面区域，无需额外请求。

### `src/routes/users.rs`

用户列表和搜索功能：

**数据模型**:

```rust
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
}
```

**主要函数**:

- **`get_all_users(pool)`**: 获取所有用户（按 ID 排序）
- **`search(Query)`**: 
  - 支持按姓名或邮箱模糊搜索
  - 使用 SQL `LIKE` 查询
  - 查询为空时返回所有用户
- **`detail(Path(id))`**: 
  - 获取指定用户详情
  - 返回用户详情卡片 HTML
  - 未找到返回 404

**搜索实现**:

```rust
// 使用 LIKE 进行模糊搜索
let search_pattern = format!("%{}%", query);
sqlx::query_as::<_, User>(
    "SELECT id, name, email FROM users 
     WHERE name LIKE ? OR email LIKE ? 
     ORDER BY id"
)
.bind(&search_pattern)
.bind(&search_pattern)
.fetch_all(&pool)
.await
```

### `src/routes/modal.rs`

模态框示例，展示如何使用 HTMX 加载动态内容到 Bootstrap 模态框。

### `templates/base.html`

基础模板，所有完整页面都继承自它：

**包含内容**:
- HTML 文档结构
- Bootstrap 5.3 CSS/JS
- HTMX 2.0 CDN
- UnoCSS CDN
- 自定义样式表
- 导航栏（带 SPA 路由）
- 页脚
- 模态框容器

**导航栏 SPA 链接示例**:

```html
<a href="/page/home" 
   hx-get="/page/home" 
   hx-target="#main-content" 
   hx-push-url="/">
   首页
</a>
```

- `href`: 降级支持（JS 禁用时）
- `hx-get`: HTMX 请求地址
- `hx-target`: 更新内容的目标元素
- `hx-push-url`: 更新浏览器 URL

### `templates/pages/`

SPA 页面内容片段，不包含 base.html 的完整结构。

- **`home.html`**: 首页欢迎内容
- **`todos.html`**: 待办列表 + 统计卡片 + 创建表单
- **`users.html`**: 搜索框 + 用户列表

### `templates/todos/`

待办事项组件：

- **`item.html`**: 
  - 单个待办项卡片
  - 复选框（切换状态）
  - 删除按钮
  - 使用 `hx-put` 和 `hx-delete`

- **`create_form.html`**: 
  - 创建表单
  - 使用 `hx-post` 提交
  - 成功后在列表顶部插入新项（`hx-swap="afterbegin"`）

- **`stats.html`**: 
  - 三张统计卡片
  - 总数、已完成、待完成
  - 使用 CountUp.js 数字动画

### `templates/users/`

用户相关组件：

- **`search_results.html`**: 
  - 用户卡片列表
  - 点击显示详情（`hx-get="/users/:id/detail"`）
  - 无结果提示

- **`detail.html`**: 
  - 用户详情卡片
  - 姓名、邮箱、ID

### `static/css/style.css`

统一的样式文件，包含：
- CSS 变量定义（颜色主题）
- 响应式布局
- 组件样式（卡片、按钮、表单）
- HTMX 过渡动画（htmx-swapping, htmx-settling）
- 自定义工具类

## 🔄 数据流

### 典型的 HTMX 请求流程

```
1. 用户交互（点击、输入等）
   ↓
2. HTMX 拦截事件，发送 HTTP 请求
   ↓
3. Axum 路由匹配对应的处理函数
   ↓
4. 处理函数执行业务逻辑（数据库查询、更新等）
   ↓
5. Askama 渲染模板为 HTML
   ↓
6. 返回 HTML 片段
   ↓
7. HTMX 接收响应，更新 DOM
   ↓
8. (可选) 使用 OOB Swap 同时更新其他区域
```

### 示例：创建待办事项

```html
<!-- 1. 用户提交表单 -->
<form hx-post="/api/todos" 
      hx-target="#todo-list" 
      hx-swap="afterbegin"
      hx-on::after-request="this.reset()">
  <input name="title" required>
  <button type="submit">添加</button>
</form>

<!-- 2. HTMX 发送 POST /api/todos -->

<!-- 3. Rust 处理函数 -->
pub async fn create(
    Extension(pool): Extension<SqlitePool>,
    Form(form): Form<CreateTodoForm>,
) -> impl IntoResponse {
    // 插入数据库
    let todo = sqlx::query_as::<_, Todo>(
        "INSERT INTO todos (title, completed) VALUES (?, 0) 
         RETURNING id, title, completed"
    )
    .bind(&form.title)
    .fetch_one(&pool)
    .await?;
    
    // 渲染待办项模板
    let todo_html = TodoItemTemplate { todo }.render()?;
    
    // 渲染统计模板
    let stats = get_stats(&pool).await?;
    let stats_html = stats.render()?;
    
    // 返回：主要内容 + OOB 更新统计
    format!(
        "{}<div id=\"todo-stats\" hx-swap-oob=\"true\">{}</div>",
        todo_html, stats_html
    )
}

<!-- 4. HTMX 接收响应 -->
<!-- 5. 在 #todo-list 顶部插入新待办项 -->
<!-- 6. OOB 更新 #todo-stats 区域 -->
```

### 示例：删除待办事项

```html
<!-- 1. 用户点击删除按钮 -->
<button hx-delete="/todos/1" 
        hx-target="#todo-1" 
        hx-swap="outerHTML">
  删除
</button>

<!-- 2. HTMX 发送 DELETE /todos/1 -->

<!-- 3. Rust 处理函数 -->
pub async fn delete(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    sqlx::query("DELETE FROM todos WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    
    let stats = get_stats(&pool).await?;
    let stats_html = stats.render()?;
    
    // 返回空内容 + OOB 更新统计
    format!(
        "<div id=\"todo-stats\" hx-swap-oob=\"true\">{}</div>",
        stats_html
    )
}

<!-- 4. HTMX 删除 #todo-1 元素（因为响应为空） -->
<!-- 5. OOB 更新 #todo-stats -->
```

## 🎨 模板继承

```
base.html (基础布局)
    ├── index.html (首页完整模板)
    ├── todos_full.html (Todos 完整页面)
    │       └── includes todos/item.html * N
    │       └── includes todos/stats.html
    │       └── includes todos/create_form.html
    │
    └── users_full.html (Users 完整页面)
            └── includes users/search_results.html
            └── dynamically loads users/detail.html

pages/ (SPA 内容片段，不继承 base.html)
    ├── home.html
    ├── todos.html
    └── users.html
```

## 📊 技术栈详解

### 后端 (Rust)

- **Axum 0.7**: 基于 Tokio 的高性能 Web 框架
- **Tokio**: 异步运行时（multi-thread）
- **Tower-HTTP**: 中间件（静态文件、tracing）
- **SQLx 0.8**: 异步 SQL 库，编译时验证
- **Serde**: JSON 序列化/反序列化
- **rust-embed 8.5**: 静态资源嵌入
- **mime_guess 2.0**: MIME 类型推断

### 模板引擎 (Askama)

- **编译时检查**: 模板错误在编译时发现，而非运行时
- **类型安全**: 变量类型在编译时验证
- **高性能**: 模板编译为 Rust 代码，零运行时开销
- **类似 Jinja2**: 熟悉的语法（if, for, extends, include）

**模板语法示例**:

```html
{% extends "base.html" %}

{% block content %}
  <h1>{{ title }}</h1>
  
  {% if items.len() > 0 %}
    {% for item in items %}
      <div>{{ item.name }}</div>
    {% endfor %}
  {% else %}
    <p>没有数据</p>
  {% endif %}
{% endblock %}
```

### 前端 (HTMX)

- **无需编写 JavaScript**: 通过 HTML 属性驱动交互
- **渐进增强**: 降级到普通 HTML 表单和链接
- **支持所有 HTTP 方法**: GET, POST, PUT, DELETE, PATCH
- **自动处理响应**: 直接更新 DOM

**常用属性**:

| 属性 | 说明 | 示例 |
|------|------|------|
| `hx-get` | 发送 GET 请求 | `hx-get="/api/data"` |
| `hx-post` | 发送 POST 请求 | `hx-post="/api/create"` |
| `hx-put` | 发送 PUT 请求 | `hx-put="/api/update/1"` |
| `hx-delete` | 发送 DELETE 请求 | `hx-delete="/api/delete/1"` |
| `hx-target` | 指定更新的元素 | `hx-target="#result"` |
| `hx-swap` | 指定更新方式 | `hx-swap="innerHTML"` |
| `hx-trigger` | 指定触发事件 | `hx-trigger="input changed delay:500ms"` |
| `hx-push-url` | 更新浏览器 URL | `hx-push-url="/todos"` |
| `hx-swap-oob` | 带外交换（OOB Swap） | `hx-swap-oob="true"` |

**hx-swap 选项**:

- `innerHTML`: 替换元素内部 HTML（默认）
- `outerHTML`: 替换整个元素
- `beforebegin`: 在元素前插入
- `afterbegin`: 在元素内部开头插入
- `beforeend`: 在元素内部末尾插入
- `afterend`: 在元素后插入
- `delete`: 删除元素
- `none`: 不交换

## 🔌 扩展点

### 添加新页面

**1. 创建模板文件**

```bash
# 创建 SPA 内容片段
touch templates/pages/about.html

# 创建完整页面（可选，用于直接访问）
touch templates/about_full.html
```

**2. 定义模板结构体** (`src/routes/mod.rs`)

```rust
#[derive(Template)]
#[template(path = "pages/about.html")]
pub struct AboutPageTemplate;

#[derive(Template)]
#[template(path = "about_full.html")]
pub struct AboutFullPageTemplate;
```

**3. 添加路由函数** (`src/routes/mod.rs`)

```rust
pub async fn about_page() -> impl IntoResponse {
    AboutFullPageTemplate
}

pub async fn page_about() -> impl IntoResponse {
    AboutPageTemplate
}
```

**4. 注册路由** (`src/main.rs`)

```rust
.route("/about", get(routes::about_page))
.route("/page/about", get(routes::page_about))
```

**5. 添加导航链接** (`templates/base.html`)

```html
<a href="/page/about" 
   hx-get="/page/about" 
   hx-target="#main-content" 
   hx-push-url="/about"
   class="nav-link">
   关于
</a>
```

### 添加新数据表

**1. 创建表结构** (`src/db.rs`)

```rust
pub async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // ... 现有表 ...
    
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;
    
    Ok(())
}
```

**2. 定义数据模型** (新建 `src/routes/posts.rs`)

```rust
#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
}
```

**3. 实现 CRUD 操作**

```rust
pub async fn get_posts(pool: &SqlitePool) -> Result<Vec<Post>, sqlx::Error> {
    sqlx::query_as::<_, Post>("SELECT id, title, content FROM posts")
        .fetch_all(pool)
        .await
}

pub async fn create(
    Extension(pool): Extension<SqlitePool>,
    Form(form): Form<CreatePostForm>,
) -> impl IntoResponse {
    // 实现逻辑
}
```

**4. 注册模块** (`src/routes/mod.rs`)

```rust
pub mod posts;
```

### 添加 WebSocket

Axum 原生支持 WebSocket：

```rust
use axum::extract::ws::{WebSocket, WebSocketUpgrade};

async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        // 处理消息
    }
}

// 注册路由
.route("/ws", get(ws_handler))
```

### 添加认证中间件

推荐使用 `tower-sessions` + `axum-login`：

```toml
[dependencies]
tower-sessions = "0.12"
axum-login = "0.15"
```

```rust
use tower_sessions::{SessionManagerLayer, MemoryStore};

let session_layer = SessionManagerLayer::new(MemoryStore::default());

let app = Router::new()
    .route("/protected", get(protected_route))
    .layer(session_layer);
```

## 📝 命名约定

- **路由函数**: 动词命名 (`list`, `create`, `update`, `delete`, `toggle`)
- **模板结构体**: `*Template` 后缀 (`IndexTemplate`, `TodoListTemplate`)
- **CSS 类**: kebab-case (`user-card`, `todo-item`, `stat-card`)
- **Rust 类型**: PascalCase (`User`, `Todo`, `Post`)
- **Rust 函数**: snake_case (`get_todos`, `create_pool`)
- **数据库表**: 复数小写 (`todos`, `users`, `posts`)

## 🚀 性能优化建议

### 编译优化

已在 `Cargo.toml` 配置：

```toml
[profile.release]
opt-level = 3          # 最高优化
lto = "fat"            # 完整 LTO
codegen-units = 1      # 最佳优化（编译较慢）
strip = true           # 剥离符号
panic = "abort"        # 减小二进制体积
```

使用 `build.sh` 启用 CPU 特定优化：

```bash
export RUSTFLAGS="-C target-cpu=native"
cargo build --release
```

### 运行时优化

1. **数据库连接池**: 已配置（最大 5 个连接）
2. **静态资源缓存**: 已配置（max-age=31536000）
3. **静态资源嵌入**: 减少磁盘 I/O
4. **编译时模板**: Askama 零运行时开销
5. **编译时 SQL**: SQLx 编译时验证，无反射开销

### 进阶优化

1. **添加 Redis 缓存**

```toml
redis = { version = "0.24", features = ["tokio-comp"] }
```

2. **启用响应压缩**

```toml
tower-http = { version = "0.6", features = ["compression-full"] }
```

```rust
use tower_http::compression::CompressionLayer;

.layer(CompressionLayer::new())
```

3. **使用 CDN** (已在模板中使用 CDN)

4. **数据库索引**

```sql
CREATE INDEX idx_todos_completed ON todos(completed);
CREATE INDEX idx_users_email ON users(email);
```

## 🐳 Docker 部署

### Dockerfile

多阶段构建，生成最小镜像：

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/htmx-rs-template /app/htmx-rs-template
WORKDIR /app
EXPOSE 3000
CMD ["./htmx-rs-template"]
```

### Docker Compose

```yaml
version: '3.8'
services:
  web:
    build: .
    ports:
      - "3000:3000"
    volumes:
      - ./data:/app/data
    environment:
      - DATABASE_URL=sqlite:///app/data/app.db?mode=rwc
      - RUST_LOG=info
```

## 📖 相关文档

- **Axum**: https://docs.rs/axum/latest/axum/
- **Askama**: https://docs.rs/askama/latest/askama/
- **HTMX**: https://htmx.org/docs/
- **SQLx**: https://docs.rs/sqlx/latest/sqlx/
- **Tokio**: https://tokio.rs/tokio/tutorial
- **Bootstrap 5**: https://getbootstrap.com/docs/5.3/
- **UnoCSS**: https://unocss.dev/

## 🎯 最佳实践

### 1. 错误处理

使用 `Result` 和 `?` 操作符：

```rust
pub async fn get_todo(pool: &SqlitePool, id: i64) -> Result<Todo, sqlx::Error> {
    sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
}
```

在路由中处理错误：

```rust
match get_todo(&pool, id).await {
    Ok(todo) => TodoTemplate { todo }.into_response(),
    Err(e) => {
        tracing::error!("获取待办失败: {}", e);
        (StatusCode::NOT_FOUND, "未找到").into_response()
    }
}
```

### 2. 日志记录

使用 `tracing` 宏：

```rust
tracing::info!("用户创建成功: {}", user.id);
tracing::warn!("数据库连接较慢");
tracing::error!("查询失败: {}", err);
tracing::debug!("请求参数: {:?}", params);
```

### 3. SQL 注入防护

始终使用参数绑定，**永远不要**拼接 SQL：

```rust
// ✅ 正确 - 使用参数绑定
sqlx::query("SELECT * FROM users WHERE name = ?")
    .bind(&user_input)
    .fetch_all(&pool)
    .await?;

// ❌ 错误 - SQL 注入风险
sqlx::query(&format!("SELECT * FROM users WHERE name = '{}'", user_input))
```

### 4. 模板复用

使用 Askama 的 `include` 功能：

```html
<!-- templates/todos/list.html -->
{% for todo in todos %}
  {% include "todos/item.html" %}
{% endfor %}
```

### 5. HTMX 事件处理

使用 HTMX 事件监听：

```html
<form hx-post="/api/todos"
      hx-on::after-request="this.reset()"
      hx-on::response-error="alert('创建失败')">
```

## 🔧 故障排查

### 编译错误

**问题**: `error: linking with 'cc' failed`

**解决**: 确保安装了 C 编译器（SQLite 依赖）

```bash
# macOS
xcode-select --install

# Ubuntu/Debian
sudo apt install build-essential

# Windows
# 安装 Visual Studio Build Tools
```

### 数据库错误

**问题**: `database is locked`

**解决**: SQLite 不支持高并发写入，考虑：
- 使用 PostgreSQL（生产环境）
- 减少连接池大小
- 使用 WAL 模式

```rust
sqlx::query("PRAGMA journal_mode=WAL")
    .execute(&pool)
    .await?;
```

### HTMX 不工作

**检查清单**:
1. HTMX CDN 是否加载成功（查看浏览器控制台）
2. 服务器是否返回正确的 HTML
3. `hx-target` 元素是否存在
4. 查看 HTMX 调试信息（`htmx.logAll()`）

```html
<script>
  htmx.logAll(); // 启用 HTMX 调试日志
</script>
```

## 📈 项目演进路线

### 第一阶段（当前）
- ✅ 基础 SPA 架构
- ✅ SQLite 数据库
- ✅ CRUD 操作
- ✅ 搜索功能

### 第二阶段（建议）
- [ ] 用户认证（登录/注册）
- [ ] 分页功能
- [ ] 数据验证（服务端 + 客户端）
- [ ] 更丰富的错误处理

### 第三阶段（高级）
- [ ] WebSocket 实时更新
- [ ] 迁移到 PostgreSQL
- [ ] 添加缓存层（Redis）
- [ ] API 文档（OpenAPI）
- [ ] 单元测试 + 集成测试

## 🤝 贡献指南

欢迎提交 Issue 和 Pull Request！

**开发流程**:
1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

**代码规范**:
- 运行 `cargo fmt` 格式化代码
- 运行 `cargo clippy` 检查警告
- 添加必要的注释和文档
