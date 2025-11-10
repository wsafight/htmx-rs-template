# HTMX + Rust SPA 模板

使用 Rust + Axum + HTMX + Bootstrap + SQLite 构建的现代化 SPA 应用模板。

## 技术栈

- **后端**: Rust + Axum (异步 Web 框架)
- **模板引擎**: Askama (编译时类型安全模板)
- **前端**: HTMX 2.0 + Bootstrap 5.3 + UnoCSS
- **数据库**: SQLite + SQLx (编译时 SQL 验证)
- **静态资源**: rust-embed (编译时嵌入)
- **日志**: tracing + tracing-subscriber

## 功能特性

- ✅ SPA 单页应用体验（HTMX 驱动）
- ✅ 无刷新页面切换和局部更新
- ✅ SQLite 数据库持久化
- ✅ 实时搜索（用户搜索）
- ✅ 完整的 CRUD 操作（待办事项）
- ✅ 统计数据实时更新（OOB Swap）
- ✅ 响应式设计（Bootstrap 5）
- ✅ 直接 URL 访问支持（/todos, /users）
- ✅ 静态资源嵌入（单一可执行文件）
- ✅ 数据库自动初始化和数据填充

## 快速开始

### 开发模式

```bash
cargo run
```

访问: http://127.0.0.1:3000

### 生产环境编译

使用提供的构建脚本（已配置优化参数）：

```bash
./build.sh
```

或手动编译：

```bash
export RUSTFLAGS="-C target-cpu=native"
cargo build --release
./target/release/htmx-rs-template
```

编译后会生成单一可执行文件，数据库 `app.db` 会自动在可执行文件同目录下创建。

## 性能优化配置

项目已配置以下 Release 优化：

- **opt-level = 3**: 最高优化级别
- **lto = "fat"**: 完整链接时优化
- **codegen-units = 1**: 单一代码生成单元（最佳优化）
- **strip = true**: 剥离调试符号（减小体积）
- **panic = "abort"**: panic 时直接中止（无栈展开开销）
- **target-cpu=native**: 针对本机 CPU 优化指令集

## 项目结构

```
.
├── src/
│   ├── main.rs              # 主程序入口，路由配置
│   ├── db.rs                # 数据库连接池、初始化、数据填充
│   └── routes/              # 路由处理模块
│       ├── mod.rs           # 路由模块入口，页面模板定义
│       ├── todos.rs         # 待办事项 CRUD + 统计
│       ├── users.rs         # 用户列表 + 搜索 + 详情
│       └── modal.rs         # 模态框示例
│
├── templates/               # Askama 模板文件
│   ├── base.html           # 基础布局（导航、CSS、JS）
│   ├── index.html          # 首页完整模板
│   ├── todos_full.html     # Todos 完整页面（直接访问）
│   ├── users_full.html     # Users 完整页面（直接访问）
│   ├── pages/              # SPA 页面内容片段
│   │   ├── home.html       # 首页内容
│   │   ├── todos.html      # 待办列表内容
│   │   └── users.html      # 用户列表内容
│   ├── todos/              # 待办相关组件
│   │   ├── item.html       # 单个待办项
│   │   ├── create_form.html # 创建表单
│   │   └── stats.html      # 统计卡片
│   ├── users/              # 用户相关组件
│   │   ├── search_results.html # 搜索结果列表
│   │   └── detail.html     # 用户详情
│   └── modal/
│       └── example.html    # 模态框内容示例
│
├── static/                  # 静态资源（编译时嵌入）
│   └── css/
│       └── style.css       # 自定义样式
│
├── Cargo.toml              # Rust 项目配置
├── askama.toml             # Askama 模板引擎配置
├── build.sh                # 优化构建脚本
├── Dockerfile              # Docker 镜像构建
├── docker-compose.yml      # Docker Compose 配置
└── README.md               # 本文档
```

## 核心功能说明

### 1. SPA 路由架构

项目支持两种访问方式：

- **SPA 内部导航**: `/page/home`, `/page/todos`, `/page/users` - 返回 HTML 片段
- **直接 URL 访问**: `/`, `/todos`, `/users` - 返回完整页面（包含 base.html）

这样既支持前端路由的 SPA 体验，又支持用户直接访问或分享特定页面 URL。

### 2. 数据库集成

- 使用 **SQLx** 进行编译时 SQL 验证
- 自动在可执行文件目录创建 `app.db`
- 启动时自动初始化表结构
- 自动填充示例数据（仅首次启动）
- 连接池管理（最大 5 个连接）

### 3. 待办事项功能

- **创建**: 表单提交创建新任务
- **删除**: 单击删除按钮
- **切换状态**: 点击复选框切换完成状态
- **实时统计**: 使用 HTMX OOB Swap 技术自动更新统计卡片（总数、已完成、待完成）

### 4. 用户管理功能

- **用户列表**: 显示所有用户
- **实时搜索**: 输入搜索词即时过滤（支持姓名和邮箱）
- **用户详情**: 点击用户显示详细信息

### 5. 静态资源嵌入

使用 `rust-embed` 在编译时将 static 目录打包进可执行文件，部署时只需一个二进制文件。

## 数据库配置

默认使用 SQLite，数据库文件位置：

- **开发模式**: 可执行文件同目录下的 `app.db`
- **环境变量**: 可通过 `DATABASE_URL` 指定自定义路径

```bash
export DATABASE_URL="sqlite:///path/to/custom.db?mode=rwc"
cargo run
```

## HTMX 关键技术

### OOB Swap (Out of Band Swap)

当操作影响多个页面区域时，使用 `hx-swap-oob` 同时更新多个元素：

```html
<!-- 主要响应：更新待办项 -->
<div id="todo-1">...</div>

<!-- OOB 响应：同时更新统计区域 -->
<div id="todo-stats" hx-swap-oob="true">
  <div class="stat-card">总数: 10</div>
  <div class="stat-card">已完成: 3</div>
  <div class="stat-card">待完成: 7</div>
</div>
```

### 常用 HTMX 属性

- `hx-get/post/put/delete`: 发送 HTTP 请求
- `hx-target`: 指定更新的 DOM 元素
- `hx-swap`: 指定更新方式（innerHTML, outerHTML, beforeend 等）
- `hx-trigger`: 指定触发事件（click, input, change 等）
- `hx-push-url`: 更新浏览器 URL（支持前进/后退）

## Docker 部署

### 构建镜像

```bash
docker build -t htmx-rs-app .
```

### 运行容器

```bash
docker run -p 3000:3000 -v $(pwd)/data:/app/data htmx-rs-app
```

### 使用 Docker Compose

```bash
docker-compose up -d
```

## 开发建议

### 添加新页面

1. 在 `templates/pages/` 创建新模板
2. 在 `src/routes/mod.rs` 添加模板结构体和路由函数
3. 在 `src/main.rs` 注册路由
4. 在 `base.html` 导航栏添加链接

### 添加新数据表

1. 在 `src/db.rs` 的 `init_db()` 添加建表语句
2. 创建对应的结构体（添加 `sqlx::FromRow` derive）
3. 在新的路由模块中实现 CRUD 操作

### 自定义样式

编辑 `static/css/style.css`，修改后重新编译即可嵌入。

## 性能特性

- **编译时模板**: Askama 模板编译为 Rust 代码，零运行时开销
- **编译时 SQL**: SQLx 在编译时验证 SQL 语句
- **静态资源嵌入**: 减少文件 I/O，提升性能
- **异步处理**: 基于 Tokio 的异步运行时
- **连接池**: SQLite 连接池管理

## 环境变量

- `DATABASE_URL`: 数据库连接字符串（可选）
- `RUST_LOG`: 日志级别配置（默认: `htmx_rs_template=debug,tower_http=debug,sqlx=info`）

示例：

```bash
export RUST_LOG="htmx_rs_template=info"
export DATABASE_URL="sqlite:///data/app.db?mode=rwc"
cargo run
```

## 访问地址

启动后访问: http://127.0.0.1:3000

- 首页: http://127.0.0.1:3000/
- 待办事项: http://127.0.0.1:3000/todos
- 用户管理: http://127.0.0.1:3000/users

## 相关文档

- [项目结构详解](PROJECT_STRUCTURE.md)
- [快速入门指南](QUICKSTART.md)
- [Bootstrap + UnoCSS 集成指南](BOOTSTRAP_UNOCSS_GUIDE.md)
- [HTMX 速查表](CHEATSHEET.md)

## License

MIT
