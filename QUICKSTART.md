# 快速入门指南

## 🚀 5分钟上手

### 1. 启动服务

```bash
cargo run
```

服务将在 `http://127.0.0.1:3000` 启动

### 2. 访问页面

打开浏览器访问以下页面：

- **首页**: http://127.0.0.1:3000
- **待办事项**: http://127.0.0.1:3000/todos
- **用户列表**: http://127.0.0.1:3000/users

### 3. 体验功能

#### 待办事项 (http://127.0.0.1:3000/todos)
1. 点击 "添加新任务" 按钮
2. 输入任务名称，点击 "添加"
3. 勾选复选框标记任务完成
4. 点击垃圾桶图标删除任务

**关键代码 - 无需刷新的交互**:
```html
<!-- 切换任务状态 -->
<input 
    type="checkbox" 
    hx-put="/todos/{{ todo.id }}/toggle"
    hx-target="#todo-{{ todo.id }}"
    hx-swap="outerHTML">

<!-- 删除任务 -->
<button 
    hx-delete="/todos/{{ todo.id }}"
    hx-target="#todo-{{ todo.id }}"
    hx-swap="outerHTML">
```

#### 用户列表 (http://127.0.0.1:3000/users)
1. 在搜索框输入用户名或邮箱
2. 实时显示匹配的用户（300ms 防抖）

**关键代码 - 实时搜索**:
```html
<input 
    type="text" 
    hx-get="/users/search"
    hx-trigger="keyup changed delay:300ms"
    hx-target="#search-results">
```

#### 模态框
1. 在首页点击 "查看模态框示例" 按钮
2. 动态加载模态框内容
3. 点击外部或关闭按钮关闭

**关键代码 - 动态加载**:
```html
<button 
    hx-get="/modal/example"
    hx-target="#modal-container"
    hx-swap="innerHTML">
```

## 📝 开发流程

### 添加新功能的步骤

#### 示例：添加一个博客页面

1. **创建路由模块** (`src/routes/blog.rs`)
```rust
use askama::Template;
use askama_axum::IntoResponse;

#[derive(Template)]
#[template(path = "blog/list.html")]
pub struct BlogListTemplate {
    pub posts: Vec<BlogPost>,
}

#[derive(Clone)]
pub struct BlogPost {
    pub id: usize,
    pub title: String,
    pub content: String,
}

pub async fn list() -> impl IntoResponse {
    let posts = vec![
        BlogPost {
            id: 1,
            title: "第一篇博客".to_string(),
            content: "这是内容...".to_string(),
        },
    ];
    BlogListTemplate { posts }
}
```

2. **注册路由模块** (`src/routes/mod.rs`)
```rust
pub mod blog;  // 添加这行
pub mod todos;
pub mod users;
pub mod modal;
```

3. **添加路由** (`src/main.rs`)
```rust
let app = Router::new()
    .route("/", get(routes::index))
    .route("/blog", get(routes::blog::list))  // 添加这行
    // ... 其他路由
```

4. **创建模板** (`templates/blog/list.html`)
```html
{% extends "../base.html" %}

{% block content %}
<h1>博客列表</h1>
<div class="blog-posts">
    {% for post in posts %}
        <article class="blog-post">
            <h2>{{ post.title }}</h2>
            <p>{{ post.content }}</p>
        </article>
    {% endfor %}
</div>
{% endblock %}
```

5. **运行并测试**
```bash
cargo run
# 访问 http://127.0.0.1:3000/blog
```

## 🎯 HTMX 常用模式

### 1. 表单提交
```html
<form hx-post="/api/submit" hx-target="#result">
    <input type="text" name="data">
    <button type="submit">提交</button>
</form>
```

### 2. 加载更多
```html
<button 
    hx-get="/api/load-more?page=2"
    hx-target="#content"
    hx-swap="beforeend">
    加载更多
</button>
```

### 3. 无限滚动
```html
<div 
    hx-get="/api/next-page"
    hx-trigger="revealed"
    hx-target="this"
    hx-swap="afterend">
</div>
```

### 4. 轮询更新
```html
<div 
    hx-get="/api/status"
    hx-trigger="every 2s"
    hx-target="this">
    当前状态: 加载中...
</div>
```

### 5. 依赖请求
```html
<select 
    hx-get="/api/cities"
    hx-trigger="change"
    hx-target="#city-select">
    <option value="1">北京</option>
</select>

<select id="city-select">
    <!-- 动态加载的选项 -->
</select>
```

## 🔧 调试技巧

### 1. 启用 HTMX 日志
在浏览器控制台输入：
```javascript
htmx.logAll()
```

### 2. 查看请求详情
在模板中添加：
```html
<div hx-get="/api/data" hx-indicator="#loading">
```

### 3. Rust 日志
设置环境变量：
```bash
RUST_LOG=debug cargo run
```

## 📚 下一步

- 阅读完整的 [README.md](README.md)
- 浏览 [HTMX 文档](https://htmx.org/docs/)
- 探索 [Axum 示例](https://github.com/tokio-rs/axum/tree/main/examples)
- 了解 [Askama 模板语法](https://djc.github.io/askama/)

## 💡 提示

- HTMX 请求会自动包含 `HX-Request: true` 头
- 可以使用 `hx-vals` 添加额外的参数
- `hx-swap` 支持多种替换策略：innerHTML, outerHTML, beforebegin, afterbegin, beforeend, afterend
- 使用 `hx-push-url="true"` 更新浏览器 URL

祝你构建愉快！🎉
