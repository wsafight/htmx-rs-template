# Docker 部署指南

本文档详细说明如何使用 Docker 和 Docker Compose 部署 HTMX-RS-Template 应用。

## 📋 目录

- [前置要求](#前置要求)
- [快速开始](#快速开始)
- [架构说明](#架构说明)
- [配置说明](#配置说明)
- [部署模式](#部署模式)
- [数据持久化](#数据持久化)
- [环境变量](#环境变量)
- [性能优化](#性能优化)
- [故障排查](#故障排查)
- [生产环境建议](#生产环境建议)

## 前置要求

确保已安装以下软件：

- **Docker**: >= 20.10
- **Docker Compose**: >= 2.0

验证安装：

```bash
docker --version
docker compose version
```

## 快速开始

### 方式一：使用 Docker Compose（推荐）

**启动应用和 Nginx 反向代理**：

```bash
# 构建并启动所有服务
docker compose up -d

# 查看日志
docker compose logs -f

# 访问应用
open http://localhost
```

**停止服务**：

```bash
docker compose down
```

**停止服务并删除数据卷**：

```bash
docker compose down -v
```

### 方式二：仅使用 Docker

**构建镜像**：

```bash
docker build -t htmx-rs-app:latest .
```

**运行容器**：

```bash
docker run -d \
  --name htmx-rs-app \
  -p 3000:3000 \
  -v htmx-data:/app/data \
  -e RUST_LOG=info \
  htmx-rs-app:latest
```

**访问应用**：

```bash
open http://localhost:3000
```

## 架构说明

### 多阶段构建

Dockerfile 使用多阶段构建来优化镜像大小：

```
Stage 1: Builder (rust:1.91.0-slim)
  ├── 安装构建依赖
  ├── 缓存 Cargo 依赖
  ├── 编译应用
  └── Strip 二进制文件

Stage 2: Runtime (debian:bookworm-slim)
  ├── 仅安装运行时依赖
  ├── 创建非 root 用户
  ├── 复制编译好的二进制
  └── 配置健康检查
```

**镜像大小对比**：

| 阶段 | 大小 |
|------|------|
| Builder 镜像 | ~1.5 GB |
| 最终运行镜像 | ~100 MB |

### 服务架构（使用 Docker Compose）

```
┌─────────────────┐
│   浏览器        │
└────────┬────────┘
         │ :80
         ▼
┌─────────────────┐
│  Nginx (Alpine) │
│  - 反向代理     │
│  - Gzip 压缩    │
│  - 静态缓存     │
│  - 安全头       │
└────────┬────────┘
         │ :3000
         ▼
┌─────────────────┐
│   Rust App      │
│  - Axum Web     │
│  - SQLite DB    │
│  - HTMX         │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Volume (DB)    │
│  /app/data      │
└─────────────────┘
```

## 配置说明

### Dockerfile 详解

**关键配置**：

```dockerfile
# 依赖缓存优化
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src
```

这一步创建虚拟 main.rs 并预编译依赖，后续修改源码时无需重新下载依赖。

**安全增强**：

```dockerfile
# 创建非 root 用户运行应用
RUN groupadd -r appuser && \
    useradd -r -g appuser -s /bin/false appuser

USER appuser
```

**健康检查**：

```dockerfile
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD ["/bin/sh", "-c", "test -f /app/data/app.db"]
```

检查数据库文件是否存在，确保应用正常初始化。

### docker-compose.yml 详解

**应用服务配置**：

```yaml
app:
  build:
    context: .
    dockerfile: Dockerfile
  container_name: htmx-rs-app
  restart: unless-stopped          # 自动重启（除非手动停止）
  environment:
    - DATABASE_URL=sqlite:///app/data/app.db?mode=rwc
    - RUST_LOG=info
  volumes:
    - app-data:/app/data           # 持久化数据库
  expose:
    - "3000"                       # 仅内部暴露，由 nginx 代理
```

**Nginx 服务配置**：

```yaml
nginx:
  image: nginx:1.27-alpine         # 轻量级 Alpine 版本
  ports:
    - "80:80"                      # 映射到主机 80 端口
  volumes:
    - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro  # 只读挂载
  depends_on:
    app:
      condition: service_healthy   # 等待 app 健康后启动
```

### Nginx 配置详解

**Gzip 压缩**：

```nginx
gzip on;
gzip_comp_level 6;
gzip_types text/plain text/css application/json application/javascript;
```

减少传输大小，提升加载速度。

**安全头**：

```nginx
add_header X-Frame-Options "SAMEORIGIN" always;
add_header X-Content-Type-Options "nosniff" always;
add_header X-XSS-Protection "1; mode=block" always;
```

防止点击劫持、MIME 类型嗅探、XSS 攻击。

**静态文件缓存**：

```nginx
location /static/ {
    proxy_pass http://app_backend;
    expires 1y;                              # 缓存 1 年
    add_header Cache-Control "public, immutable";
}
```

**HTMX 支持**：

```nginx
proxy_buffering off;  # 禁用缓冲，支持实时更新
```

## 部署模式

### 开发模式

使用绑定挂载实现热重载：

```yaml
# docker-compose.dev.yml
services:
  app:
    build:
      context: .
      target: builder  # 使用 builder 阶段
    volumes:
      - ./src:/app/src:ro          # 挂载源码（只读）
      - ./templates:/app/templates:ro
      - ./static:/app/static:ro
      - app-data:/app/data
    command: cargo watch -x run    # 使用 cargo-watch
```

启动开发环境：

```bash
docker compose -f docker-compose.dev.yml up
```

### 生产模式

使用默认配置即可：

```bash
docker compose up -d
```

### 多环境配置

**创建环境特定的配置文件**：

```bash
# 开发环境
docker compose -f docker-compose.yml -f docker-compose.dev.yml up

# 生产环境
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

**docker-compose.prod.yml 示例**：

```yaml
version: '3.8'

services:
  app:
    environment:
      - RUST_LOG=warn,htmx_rs_template=info
    deploy:
      resources:
        limits:
          cpus: '1'
          memory: 512M
        reservations:
          cpus: '0.5'
          memory: 256M
    restart: always

  nginx:
    ports:
      - "443:443"  # HTTPS
    volumes:
      - ./nginx/ssl:/etc/nginx/ssl:ro
      - ./nginx/nginx.prod.conf:/etc/nginx/nginx.conf:ro
```

## 数据持久化

### Volume 管理

**查看数据卷**：

```bash
docker volume ls
```

**检查数据卷详情**：

```bash
docker volume inspect htmx-rs-template_app-data
```

**备份数据库**：

```bash
# 方法 1: 从容器复制
docker cp htmx-rs-app:/app/data/app.db ./backups/app-$(date +%Y%m%d).db

# 方法 2: 使用 Volume 备份
docker run --rm \
  -v htmx-rs-template_app-data:/data \
  -v $(pwd)/backups:/backup \
  alpine tar czf /backup/app-data-$(date +%Y%m%d).tar.gz /data
```

**恢复数据库**：

```bash
# 停止应用
docker compose down

# 恢复数据
docker run --rm \
  -v htmx-rs-template_app-data:/data \
  -v $(pwd)/backups:/backup \
  alpine tar xzf /backup/app-data-20250110.tar.gz -C /

# 启动应用
docker compose up -d
```

### 绑定挂载（生产环境推荐）

修改 `docker-compose.yml` 使用主机目录：

```yaml
services:
  app:
    volumes:
      - ./data:/app/data  # 使用主机目录
```

优势：
- 更容易备份和迁移
- 可直接访问数据库文件
- 适合生产环境

## 环境变量

### 可用环境变量

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| `DATABASE_URL` | `sqlite:///app/data/app.db?mode=rwc` | 数据库连接字符串 |
| `RUST_LOG` | `info` | 日志级别 (trace/debug/info/warn/error) |
| `BIND_ADDRESS` | `127.0.0.1:3000` | 监听地址和端口 |

### 设置环境变量

**方式一：docker-compose.yml**

```yaml
services:
  app:
    environment:
      - RUST_LOG=debug
      - DATABASE_URL=sqlite:///app/data/custom.db
```

**方式二：.env 文件**

创建 `.env` 文件：

```bash
RUST_LOG=debug
DATABASE_URL=sqlite:///app/data/app.db?mode=rwc
```

Docker Compose 会自动加载。

**方式三：命令行**

```bash
docker run -e RUST_LOG=debug -e DATABASE_URL=... htmx-rs-app
```

### 日志级别配置

**详细调试**：

```bash
RUST_LOG=htmx_rs_template=trace,tower_http=debug,sqlx=debug
```

**生产环境**（推荐）：

```bash
RUST_LOG=warn,htmx_rs_template=info
```

## 性能优化

### 镜像构建优化

**启用 BuildKit**：

```bash
export DOCKER_BUILDKIT=1
docker build -t htmx-rs-app .
```

**使用构建缓存**：

```bash
# 使用远程缓存
docker build \
  --cache-from htmx-rs-app:latest \
  -t htmx-rs-app:latest .
```

**多平台构建**：

```bash
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t htmx-rs-app:latest \
  --push .
```

### 运行时优化

**资源限制**：

```yaml
services:
  app:
    deploy:
      resources:
        limits:
          cpus: '1'
          memory: 512M
        reservations:
          cpus: '0.25'
          memory: 128M
```

**网络优化**：

```yaml
networks:
  app-network:
    driver: bridge
    driver_opts:
      com.docker.network.driver.mtu: 1500
```

### Nginx 性能调优

```nginx
# nginx.conf
worker_processes auto;
worker_connections 2048;

# 启用 HTTP/2
listen 443 ssl http2;

# 启用 TCP Fast Open
listen 80 fastopen=256;
```

## 故障排查

### 查看日志

**所有服务日志**：

```bash
docker compose logs -f
```

**特定服务日志**：

```bash
docker compose logs -f app
docker compose logs -f nginx
```

**实时日志（最近 100 行）**：

```bash
docker compose logs --tail=100 -f app
```

### 进入容器调试

```bash
# 进入应用容器
docker compose exec app /bin/sh

# 进入 Nginx 容器
docker compose exec nginx /bin/sh
```

### 常见问题

#### 1. 容器无法启动

**问题**: `Error: database is locked`

**原因**: 多个进程访问 SQLite

**解决**:

```bash
# 停止所有容器
docker compose down

# 删除数据库锁文件
docker volume rm htmx-rs-template_app-data

# 重新启动
docker compose up -d
```

#### 2. 数据库未初始化

**问题**: 应用启动但无数据

**解决**:

```bash
# 检查数据库文件
docker compose exec app ls -lh /app/data/

# 查看应用日志
docker compose logs app | grep "数据库"

# 手动删除并重启（会重新初始化）
docker compose exec app rm /app/data/app.db
docker compose restart app
```

#### 3. Nginx 502 Bad Gateway

**问题**: Nginx 无法连接到应用

**解决**:

```bash
# 检查应用是否运行
docker compose ps

# 检查应用健康状态
docker inspect htmx-rs-app | grep -A 5 Health

# 检查网络连接
docker compose exec nginx ping app

# 重启服务
docker compose restart
```

#### 4. 端口已被占用

**问题**: `Bind for 0.0.0.0:80 failed: port is already allocated`

**解决**:

```bash
# 查找占用端口的进程
lsof -i :80

# 修改端口映射
# docker-compose.yml
ports:
  - "8080:80"  # 使用 8080 端口
```

### 健康检查

**查看健康状态**：

```bash
docker compose ps
docker inspect --format='{{.State.Health.Status}}' htmx-rs-app
```

**手动测试健康检查**：

```bash
docker compose exec app test -f /app/data/app.db && echo "健康" || echo "不健康"
```

## 生产环境建议

### 1. 使用 HTTPS

**安装 Certbot**：

```bash
docker compose -f docker-compose.prod.yml up -d
```

**docker-compose.prod.yml**：

```yaml
services:
  certbot:
    image: certbot/certbot
    volumes:
      - ./nginx/ssl:/etc/letsencrypt
      - ./nginx/webroot:/var/www/certbot
    command: certonly --webroot -w /var/www/certbot --email your@email.com -d yourdomain.com --agree-tos
```

**Nginx SSL 配置**：

```nginx
server {
    listen 443 ssl http2;
    server_name yourdomain.com;

    ssl_certificate /etc/nginx/ssl/live/yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/nginx/ssl/live/yourdomain.com/privkey.pem;
    
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;
}
```

### 2. 自动重启

```yaml
services:
  app:
    restart: always  # 总是重启
```

### 3. 资源限制

```yaml
services:
  app:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 1G
        reservations:
          cpus: '0.5'
          memory: 256M
```

### 4. 日志管理

```yaml
services:
  app:
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

### 5. 监控和告警

使用 Prometheus + Grafana：

```yaml
# docker-compose.monitoring.yml
services:
  prometheus:
    image: prom/prometheus
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    ports:
      - "9090:9090"

  grafana:
    image: grafana/grafana
    ports:
      - "3001:3000"
```

### 6. 定期备份

创建备份脚本 `backup.sh`：

```bash
#!/bin/bash
BACKUP_DIR="./backups"
DATE=$(date +%Y%m%d_%H%M%S)

mkdir -p $BACKUP_DIR

docker cp htmx-rs-app:/app/data/app.db $BACKUP_DIR/app-$DATE.db

# 保留最近 7 天的备份
find $BACKUP_DIR -name "app-*.db" -mtime +7 -delete

echo "备份完成: $BACKUP_DIR/app-$DATE.db"
```

添加到 crontab：

```bash
# 每天凌晨 2 点备份
0 2 * * * /path/to/backup.sh
```

### 7. 使用 Watchtower 自动更新

```yaml
services:
  watchtower:
    image: containrrr/watchtower
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    command: --interval 3600  # 每小时检查更新
```

## 部署检查清单

部署前请确认：

- [ ] 环境变量配置正确
- [ ] 数据卷已配置持久化
- [ ] 端口映射无冲突
- [ ] 日志级别设置合理
- [ ] 健康检查正常工作
- [ ] Nginx 配置已测试
- [ ] SSL 证书已配置（生产环境）
- [ ] 备份策略已实施
- [ ] 资源限制已设置
- [ ] 监控和告警已配置

## 参考资源

- [Docker 官方文档](https://docs.docker.com/)
- [Docker Compose 文档](https://docs.docker.com/compose/)
- [Nginx 官方文档](https://nginx.org/en/docs/)
- [Rust Docker 最佳实践](https://docs.docker.com/language/rust/)

## 许可证

MIT
