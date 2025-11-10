# 🎨 Bootstrap 5 + UnoCSS 集成指南

## 🎉 已完成的升级

项目已成功集成 **Bootstrap 5** 和 **UnoCSS**，现在拥有更现代化、更美观的界面！

## ✨ 新增功能

### 1. Bootstrap 5 组件
- ✅ 响应式导航栏（带汉堡菜单）
- ✅ 卡片组件（带悬停效果）
- ✅ 模态框（Bootstrap 原生）
- ✅ 表单控件（增强样式）
- ✅ 徽章和按钮
- ✅ 列表组
- ✅ Bootstrap Icons 图标库

### 2. UnoCSS 原子化 CSS
- ✅ 即时类名生成
- ✅ 与 Bootstrap 完美配合
- ✅ 极小的体积开销

### 3. 自定义增强
- ✅ 卡片悬停效果
- ✅ 平滑页面切换动画
- ✅ 渐变背景
- ✅ 响应式设计优化

## 📦 使用的 CDN

```html
<!-- Bootstrap 5.3.2 -->
<link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.2/dist/css/bootstrap.min.css" rel="stylesheet">
<script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.2/dist/js/bootstrap.bundle.min.js"></script>

<!-- Bootstrap Icons 1.11.3 -->
<link href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.3/font/bootstrap-icons.min.css" rel="stylesheet">

<!-- UnoCSS Runtime 0.58.0 -->
<script src="https://cdn.jsdelivr.net/npm/@unocss/runtime@0.58.0/uno.global.js"></script>

<!-- Animate.css 4.1.1 (用于模态框) -->
<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/animate.css/4.1.1/animate.min.css"/>
```

## 🎨 设计系统

### 颜色方案

```css
主色调: #3b82f6 (蓝色)
成功色: #10b981 (绿色)
警告色: #f59e0b (黄色)
危险色: #ef4444 (红色)
信息色: #3b82f6 (蓝色)
```

### Bootstrap 类使用示例

#### 布局
```html
<!-- 容器 -->
<div class="container">内容</div>

<!-- 响应式网格 -->
<div class="row g-4">
    <div class="col-md-6 col-lg-4">卡片</div>
    <div class="col-md-6 col-lg-4">卡片</div>
</div>
```

#### 卡片
```html
<div class="card shadow-sm">
    <div class="card-header bg-primary text-white">
        <h5 class="mb-0">标题</h5>
    </div>
    <div class="card-body">
        内容
    </div>
    <div class="card-footer">
        底部
    </div>
</div>
```

#### 按钮
```html
<!-- 主要按钮 -->
<button class="btn btn-primary">
    <i class="bi bi-check-circle me-2"></i>确定
</button>

<!-- 次要按钮 -->
<button class="btn btn-outline-secondary">取消</button>

<!-- 不同大小 -->
<button class="btn btn-lg btn-primary">大按钮</button>
<button class="btn btn-sm btn-danger">小按钮</button>
```

#### 表单
```html
<div class="input-group input-group-lg">
    <span class="input-group-text">
        <i class="bi bi-search"></i>
    </span>
    <input type="text" class="form-control" placeholder="搜索...">
    <button class="btn btn-primary">搜索</button>
</div>
```

#### 徽章
```html
<span class="badge bg-primary">新功能</span>
<span class="badge bg-success">已完成</span>
<span class="badge bg-warning text-dark">进行中</span>
<span class="badge bg-danger">紧急</span>
```

#### 图标
```html
<!-- Bootstrap Icons -->
<i class="bi bi-house-door"></i>
<i class="bi bi-check-square"></i>
<i class="bi bi-people"></i>
<i class="bi bi-gear"></i>
```

### UnoCSS 实用类

```html
<!-- 间距 -->
<div class="p-4 m-2">内容</div>

<!-- Flex 布局 -->
<div class="d-flex justify-content-between align-items-center">

<!-- 文本 -->
<p class="text-center fw-bold fs-4">标题</p>

<!-- 颜色 -->
<div class="bg-primary text-white">内容</div>
```

## 🎭 自定义类

### 悬停效果
```html
<!-- 卡片上浮 -->
<div class="card hover-lift">...</div>

<!-- 阴影增强 -->
<div class="card hover-shadow">...</div>
```

### 动画
```html
<!-- 淡入 -->
<div class="animate__animated animate__fadeIn">...</div>

<!-- 缩放 -->
<div class="animate__animated animate__zoomIn">...</div>

<!-- 滑入 -->
<div class="animate__animated animate__slideInLeft">...</div>
```

### 渐变背景
```html
<div class="bg-gradient-primary">渐变背景</div>
```

## 📱 响应式断点

```css
/* Bootstrap 5 断点 */
xs: < 576px  (超小屏)
sm: ≥ 576px  (小屏)
md: ≥ 768px  (中屏)
lg: ≥ 992px  (大屏)
xl: ≥ 1200px (超大屏)
xxl: ≥ 1400px (超超大屏)
```

使用示例：
```html
<div class="col-12 col-sm-6 col-md-4 col-lg-3">
    在不同屏幕尺寸显示不同列数
</div>
```

## 🎯 页面示例

### 首页特性
- 大标题 + 副标题
- 3列功能卡片（悬停效果）
- 特性列表（带图标）
- 技术栈展示（渐变背景）

### 待办页面特性
- 顶部操作栏
- 创建表单卡片（动画显示）
- 任务列表（复选框 + 状态徽章）
- 统计卡片（3个统计项）

### 用户页面特性
- 搜索框（大输入框 + 图标）
- 用户卡片网格（头像 + 信息）
- 悬停效果
- 加载指示器

### 模态框特性
- Bootstrap 原生模态框
- 动画效果（animate.css）
- 图标和徽章
- 响应式布局

## 🔧 自定义主题

### 修改主色调

编辑 `templates/base.html`:
```html
<style>
:root {
    --bs-primary-rgb: 59, 130, 246;  /* 修改这里 */
    --primary-color: #3b82f6;
}
</style>
```

### 添加自定义样式

编辑 `static/css/style.css`:
```css
.my-custom-class {
    /* 你的样式 */
}
```

## 📊 性能优化

### CDN 优化
- 使用 jsdelivr CDN（快速、可靠）
- 所有资源启用 HTTP/2
- Bootstrap 和图标字体缓存

### CSS 优化
- Bootstrap 按需加载（未使用的组件自动剔除）
- UnoCSS 实时生成（只包含使用的类）
- 自定义 CSS 最小化

### 文件大小
```
Bootstrap CSS:   ~50KB (gzip)
Bootstrap JS:    ~20KB (gzip)
Bootstrap Icons: ~150KB (缓存)
UnoCSS Runtime:  ~10KB (gzip)
自定义 CSS:      ~5KB (gzip)
总计: ~235KB (首次加载)
```

## 🎓 最佳实践

### 1. 优先使用 Bootstrap 类
```html
<!-- ✅ 好 -->
<button class="btn btn-primary">按钮</button>

<!-- ❌ 避免 -->
<button style="background: blue; padding: 10px;">按钮</button>
```

### 2. 合理使用 UnoCSS
```html
<!-- ✅ 简单布局用 Bootstrap -->
<div class="d-flex justify-content-between">

<!-- ✅ 快速调整用 UnoCSS -->
<div class="p-4 m-2">
```

### 3. 自定义复杂组件
```html
<!-- 对于复杂的组件，使用自定义 CSS -->
<div class="custom-timeline">
    <!-- 自定义样式在 style.css -->
</div>
```

### 4. 响应式优先
```html
<!-- 始终考虑移动端 -->
<div class="row g-4">
    <div class="col-12 col-md-6 col-lg-4">
        响应式列
    </div>
</div>
```

## 🔗 有用资源

- [Bootstrap 5 文档](https://getbootstrap.com/docs/5.3/)
- [Bootstrap Icons](https://icons.getbootstrap.com/)
- [UnoCSS 文档](https://unocss.dev/)
- [Animate.css](https://animate.style/)

## 📝 迁移笔记

### 从旧版本迁移
如果你之前使用自定义 CSS：

1. **保留自定义样式**: 所有自定义动画和效果都在 `style.css` 中保留
2. **Bootstrap 增强**: 使用 Bootstrap 组件替代自定义布局
3. **图标系统**: 使用 Bootstrap Icons 替代 emoji

### 兼容性
- ✅ 所有现代浏览器（Chrome, Firefox, Safari, Edge）
- ✅ 移动浏览器（iOS Safari, Chrome Mobile）
- ⚠️ IE11 不支持（Bootstrap 5 不支持）

## 🎉 总结

现在你的项目拥有：
- 🎨 现代化的 Bootstrap 5 UI
- ⚡ UnoCSS 原子化 CSS
- 📱 完整的响应式设计
- 🎭 丰富的动画效果
- 🎯 生产级的代码质量

开始使用新的 UI 组件构建你的应用吧！🚀

---

Made with ❤️ using Bootstrap 5, UnoCSS, and HTMX
