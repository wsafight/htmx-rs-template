# HTMX + Rust SPA 模板

使用 Rust + Axum + HTMX + Bootstrap + SQLite 构建的现代化 SPA 应用模板。

## 技术栈

- **后端**: Rust + Axum
- **模板引擎**: Askama
- **前端**: HTMX 2.0 + Bootstrap 5.3 + UnoCSS
- **数据库**: SQLite (SQLx)
- **动画**: CountUp.js

## 功能特性

- ✅ SPA 单页应用体验
- ✅ 无刷新页面切换
- ✅ 数据库持久化
- ✅ 实时搜索
- ✅ CRUD 操作
- ✅ 模态框示例
- ✅ 响应式设计
- ✅ 数字动画效果

## 快速开始

### 开发模式

```bash
cargo run
```

### 性能优化编译

```bash
./build.sh
```

或手动编译：

```bash
export RUSTFLAGS="-C target-cpu=native"
cargo build --release
./target/release/htmx-rs-template
```

## 性能优化配置

项目已配置以下性能优化：

- **opt-level = 3**: 最高优化级别
- **lto = "fat"**: 完整链接时优化
- **codegen-units = 1**: 单一代码生成单元
- **strip = true**: 剥离调试符号
- **panic = "abort"**: panic 时直接中止
- **target-cpu=native**: 针对本机 CPU 优化

## 项目结构

```
.
├── src/
│   ├── main.rs           # 主程序入口
│   ├── db.rs             # 数据库模块
│   └── routes/           # 路由处理
│       ├── mod.rs
│       ├── todos.rs      # 待办事项
│       ├── users.rs      # 用户管理
│       └── modal.rs      # 模态框
├── templates/            # 模板文件
│   ├── base.html         # 基础布局
│   ├── pages/            # SPA 页面片段
│   ├── todos/            # 待办组件
│   └── users/            # 用户组件
├── static/               # 静态资源
└── Cargo.toml
```

## 访问地址

启动后访问: http://127.0.0.1:3000

## License

MIT
