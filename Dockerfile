# 构建阶段
FROM rust:1.91.0-slim as builder

WORKDIR /app

# 复制依赖文件
COPY Cargo.toml Cargo.lock ./

# 创建一个虚拟的 main.rs 来缓存依赖
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 复制源代码和模板
COPY src ./src
COPY templates ./templates
COPY static ./static

# 构建应用
RUN cargo build --release

# 运行阶段
FROM debian:bookworm-slim

WORKDIR /app

# 安装运行时依赖
RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# 从构建阶段复制编译好的二进制文件
COPY --from=builder /app/target/release/htmx-rs-template /app/htmx-rs-template

# 暴露端口
EXPOSE 3000

# 运行应用
CMD ["/app/htmx-rs-template"]
