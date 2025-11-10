# 🚀 HTMX + Rust SPA 速查表

## 快速命令

```bash
# 构建项目
cargo build

# 运行服务器
cargo run

# 访问应用
open http://127.0.0.1:3000

# 查看日志
RUST_LOG=debug cargo run
```

## HTMX 核心属性

### 基础请求
```html
<!-- GET 请求 -->
<button hx-get="/api/data">加载</button>

<!-- POST 请求 -->
<form hx-post="/api/submit">...</form>

<!-- PUT 请求 -->
<button hx-put="/api/update/1">更新</button>

<!-- DELETE 请求 -->
<button hx-delete="/api/delete/1">删除</button>
```

### 目标和交换
```html
<!-- 指定更新目标 -->
hx-target="#result"          <!-- ID 选择器 -->
hx-target=".list"            <!-- 类选择器 -->
hx-target="this"             <!-- 当前元素 -->
hx-target="closest .card"    <!-- 最近的父元素 -->

<!-- 交换策略 -->
hx-swap="innerHTML"          <!-- 替换内部 HTML（默认） -->
hx-swap="outerHTML"          <!-- 替换整个元素 -->
hx-swap="beforebegin"        <!-- 插入到元素之前 -->
hx-swap="afterbegin"         <!-- 插入到开头 -->
hx-swap="beforeend"          <!-- 插入到末尾 -->
hx-swap="afterend"           <!-- 插入到元素之后 -->
hx-swap="delete"             <!-- 删除元素 -->
hx-swap="none"               <!-- 不交换 -->
```

### 触发器
```html
<!-- 点击触发（默认） -->
<button hx-get="/data">点击</button>

<!-- 改变时触发 -->
<input hx-get="/search" hx-trigger="change">

<!-- 键盘输入触发 -->
<input hx-get="/search" hx-trigger="keyup">

<!-- 延迟触发 -->
<input hx-get="/search" hx-trigger="keyup changed delay:300ms">

<!-- 加载时触发 -->
<div hx-get="/data" hx-trigger="load">

<!-- 滚动到可见时触发 -->
<div hx-get="/more" hx-trigger="revealed">

<!-- 轮询 -->
<div hx-get="/status" hx-trigger="every 2s">
```

### SPA 导航
```html
<!-- SPA 链接 -->
<a href="/page" 
   hx-get="/page/content"
   hx-target="#main"
   hx-push-url="/page">
   导航
</a>

<!-- 启用 boost（自动 AJAX） -->
<body hx-boost="true">
```

## Rust 路由模式

### 基础路由
```rust
use axum::{Router, routing::get};

let app = Router::new()
    .route("/", get(handler))
    .route("/path", get(handler))
    .route("/path/:id", get(handler_with_id));
```

### 路径参数
```rust
use axum::extract::Path;

async fn handler(Path(id): Path<usize>) -> String {
    format!("ID: {}", id)
}
```

### 查询参数
```rust
use axum::extract::Query;
use serde::Deserialize;

#[derive(Deserialize)]
struct Params {
    q: String,
}

async fn handler(Query(params): Query<Params>) -> String {
    params.q
}
```

### 表单数据
```rust
use axum::Form;
use serde::Deserialize;

#[derive(Deserialize)]
struct FormData {
    name: String,
}

async fn handler(Form(data): Form<FormData>) -> String {
    data.name
}
```

## Askama 模板语法

### 变量
```html
{{ variable }}
{{ user.name }}
{{ items[0] }}
```

### 条件
```html
{% if user.active %}
    活跃用户
{% else %}
    非活跃
{% endif %}
```

### 循环
```html
{% for item in items %}
    <li>{{ item }}</li>
{% endfor %}
```

### 继承
```html
<!-- base.html -->
<!DOCTYPE html>
<html>
<body>
    {% block content %}{% endblock %}
</body>
</html>

<!-- page.html -->
{% extends "base.html" %}
{% block content %}
    <h1>内容</h1>
{% endblock %}
```

### 包含
```html
{% include "partials/header.html" %}
```

## CSS 类和动画

### HTMX 类
```css
/* 请求进行中 */
.htmx-request { }

/* 内容交换中 */
.htmx-swapping { }

/* 内容稳定中 */
.htmx-settling { }

/* 新添加的元素 */
.htmx-added { }
```

### 自定义动画
```css
@keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
}

.fade-in {
    animation: fadeIn 0.3s;
}
```

## 调试技巧

### HTMX 日志
```javascript
// 在浏览器控制台
htmx.logAll()
```

### HTMX 事件
```javascript
// 请求前
document.body.addEventListener('htmx:beforeRequest', (e) => {
    console.log('请求前:', e.detail);
});

// 请求后
document.body.addEventListener('htmx:afterRequest', (e) => {
    console.log('请求后:', e.detail);
});

// 交换前
document.body.addEventListener('htmx:beforeSwap', (e) => {
    console.log('交换前:', e.detail);
});

// 交换后
document.body.addEventListener('htmx:afterSwap', (e) => {
    console.log('交换后:', e.detail);
});
```

### Rust 日志
```bash
# 启用调试日志
RUST_LOG=debug cargo run

# 仅应用日志
RUST_LOG=htmx_rs_template=debug cargo run

# 详细日志
RUST_LOG=trace cargo run
```

## 常见模式

### 实时搜索
```html
<input 
    type="text"
    hx-get="/search"
    hx-trigger="keyup changed delay:300ms"
    hx-target="#results">
```

### 无限滚动
```html
<div hx-get="/next" hx-trigger="revealed" hx-swap="afterend">
    加载更多...
</div>
```

### 表单验证
```html
<input 
    name="email"
    hx-post="/validate"
    hx-trigger="blur"
    hx-target="#error">
```

### 删除确认
```html
<button 
    hx-delete="/item/1"
    hx-confirm="确定删除？">
    删除
</button>
```

### 加载指示器
```html
<button hx-get="/data" hx-indicator="#spinner">
    加载
</button>
<div id="spinner" class="htmx-indicator">
    加载中...
</div>
```

## 项目结构速览

```
├── src/
│   ├── main.rs           # 入口
│   └── routes/           # 路由
├── templates/
│   ├── base.html         # 基础布局
│   ├── pages/            # SPA 页面
│   └── components/       # 组件
└── static/
    └── css/              # 样式
```

## 有用链接

- 📖 [README.md](README.md) - 项目概述
- 🚀 [QUICKSTART.md](QUICKSTART.md) - 快速开始
- 🏗️ [SPA_GUIDE.md](SPA_GUIDE.md) - SPA 架构
- 📁 [PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md) - 项目结构
- 🎉 [SUMMARY.md](SUMMARY.md) - 项目总结

---

💡 **提示**: 保持这个文件打开，随时查阅！
