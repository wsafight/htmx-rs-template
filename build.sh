#!/bin/bash

# 性能优化编译脚本

echo "🚀 开始性能优化编译..."

# 设置 RUSTFLAGS 环境变量以启用更多优化
export RUSTFLAGS="-C target-cpu=native"

# 清理之前的构建
cargo clean

# 使用 release 模式编译，启用所有优化
cargo build --release

echo "✅ 编译完成！二进制文件位于: target/release/htmx-rs-template"
echo "📦 文件大小:"
ls -lh target/release/htmx-rs-template

echo ""
echo "🎯 运行命令:"
echo "./target/release/htmx-rs-template"
