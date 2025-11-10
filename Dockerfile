# 构建阶段
FROM rust:1.91.0-slim as builder

WORKDIR /app

# 安装构建依赖
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# 复制依赖文件
COPY Cargo.toml Cargo.lock ./
COPY askama.toml ./

# 创建一个虚拟的 main.rs 来缓存依赖
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 复制源代码和资源文件
COPY src ./src
COPY templates ./templates
COPY static ./static

# 构建应用（使用 release 优化）
RUN cargo build --release && \
    strip /app/target/release/htmx-rs-template

# 运行阶段 - 使用最小化镜像
FROM debian:bookworm-slim

WORKDIR /app

# 安装运行时依赖
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 && \
    rm -rf /var/lib/apt/lists/*

# 创建非 root 用户
RUN groupadd -r appuser && \
    useradd -r -g appuser -s /bin/false appuser && \
    mkdir -p /app/data && \
    chown -R appuser:appuser /app

# 从构建阶段复制编译好的二进制文件
COPY --from=builder /app/target/release/htmx-rs-template /app/htmx-rs-template

# 切换到非 root 用户
USER appuser

# 设置数据库路径环境变量
ENV DATABASE_URL=sqlite:///app/data/app.db?mode=rwc
ENV RUST_LOG=htmx_rs_template=info,tower_http=info,sqlx=warn

# 暴露端口
EXPOSE 3000

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/bin/sh", "-c", "test -f /app/data/app.db"]

# 运行应用
CMD ["/app/htmx-rs-template"]
