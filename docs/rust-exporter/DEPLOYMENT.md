# Rust Exporter 部署指南

## 📋 目录

1. [前置要求](#前置要求)
2. [快速部署 (Docker)](#快速部署-docker)
3. [本地开发部署](#本地开发部署)
4. [生产环境部署](#生产环境部署)
5. [验证部署](#验证部署)
6. [监控和维护](#监控和维护)
7. [故障排查](#故障排查)

---

## 前置要求

### 必需
- Docker 和 Docker Compose
- PostgreSQL 数据库访问权限
- 端口 8001 和 8002 可用

### 可选
- Rust 1.75+ (本地开发)
- Git (克隆代码)

---

## 快速部署 (Docker)

### 方法 1: 使用现有的 docker-compose.yml

#### 步骤 1: 配置环境变量

```bash
# 编辑 .env 文件
cd /path/to/channel-health-monitoring
cp .env.example .env
vim .env
```

在 `.env` 中配置：

```bash
# Grafana 配置
GRAFANA_ADMIN_USER=admin
GRAFANA_ADMIN_PASSWORD=admin

# 数据库连接信息
DB_HOST=your-database-host
DB_PORT=5432
DB_NAME=your-database-name
DB_USER=your-database-user
DB_PASSWORD=your-database-password
```

#### 步骤 2: 构建并启动

```bash
# 构建 Rust exporter
docker compose build rust-exporter

# 启动服务
docker compose up -d rust-exporter

# 查看日志
docker compose logs -f rust-exporter
```

#### 步骤 3: 验证

```bash
# 检查服务状态
docker compose ps

# 测试健康检查
curl http://localhost:8001/health
# 应该返回: OK

# 查看指标
curl http://localhost:8001/metrics
# 应该看到 Prometheus 格式的指标
```

### 方法 2: 单独运行 Rust Exporter

```bash
# 构建镜像
cd rust-exporter
docker build -t rust-exporter:latest .

# 运行容器
docker run -d \
  --name rust-exporter \
  -p 8001:8001 \
  -p 8002:8002 \
  -e DATABASE_URL="postgres://user:pass@host:5432/dbname" \
  rust-exporter:latest

# 查看日志
docker logs -f rust-exporter
```

---

## 本地开发部署

### 步骤 1: 安装 Rust

```bash
# 安装 Rust (如果还没有)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 验证安装
rustc --version
cargo --version
```

### 步骤 2: 克隆代码

```bash
git clone https://github.com/DaneelOlivaw1/channel-health-monitoring.git
cd channel-health-monitoring/rust-exporter
```

### 步骤 3: 配置环境变量

```bash
# 创建 .env 文件
cat > .env << EOF
DATABASE_URL=postgres://user:password@host:5432/database
EOF
```

### 步骤 4: 运行

```bash
# 开发模式 (带调试信息)
cargo run

# 或者发布模式 (优化性能)
cargo run --release
```

### 步骤 5: 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test --test availability_collector_tests

# 查看测试覆盖率
cargo test -- --nocapture
```

---

## 生产环境部署

### 选项 1: Docker Compose (推荐)

#### 完整的监控栈部署

```bash
# 1. 克隆仓库
git clone https://github.com/DaneelOlivaw1/channel-health-monitoring.git
cd channel-health-monitoring

# 2. 配置环境变量
cp .env.example .env
vim .env  # 编辑数据库连接信息

# 3. 启动所有服务
docker compose up -d

# 4. 验证服务
docker compose ps
```

这将启动：
- ✅ Rust Exporter (8001, 8002)
- ✅ Python Exporter (8003, 8004) - 可选，用于对比
- ✅ Prometheus (9090)
- ✅ Grafana (3000)

#### 只部署 Rust Exporter

```bash
# 只启动 Rust exporter
docker compose up -d rust-exporter

# 或者指定服务
docker compose up -d rust-exporter prometheus grafana
```

### 选项 2: Kubernetes 部署

#### 创建 Kubernetes 配置

```yaml
# rust-exporter-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rust-exporter
  labels:
    app: rust-exporter
spec:
  replicas: 2
  selector:
    matchLabels:
      app: rust-exporter
  template:
    metadata:
      labels:
        app: rust-exporter
    spec:
      containers:
      - name: rust-exporter
        image: rust-exporter:latest
        ports:
        - containerPort: 8001
          name: metrics
        - containerPort: 8002
          name: admin
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: rust-exporter-secret
              key: database-url
        resources:
          requests:
            memory: "20Mi"
            cpu: "100m"
          limits:
            memory: "50Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8001
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /health
            port: 8001
          initialDelaySeconds: 5
          periodSeconds: 10
---
apiVersion: v1
kind: Service
metadata:
  name: rust-exporter
  labels:
    app: rust-exporter
spec:
  type: ClusterIP
  ports:
  - port: 8001
    targetPort: 8001
    name: metrics
  - port: 8002
    targetPort: 8002
    name: admin
  selector:
    app: rust-exporter
---
apiVersion: v1
kind: Secret
metadata:
  name: rust-exporter-secret
type: Opaque
stringData:
  database-url: "postgres://user:password@host:5432/database"
```

#### 部署到 Kubernetes

```bash
# 应用配置
kubectl apply -f rust-exporter-deployment.yaml

# 查看状态
kubectl get pods -l app=rust-exporter
kubectl get svc rust-exporter

# 查看日志
kubectl logs -f deployment/rust-exporter

# 端口转发 (本地测试)
kubectl port-forward svc/rust-exporter 8001:8001
```

### 选项 3: 系统服务 (Systemd)

#### 创建 systemd 服务文件

```bash
# 编译发布版本
cd rust-exporter
cargo build --release

# 复制二进制文件
sudo cp target/release/rust-exporter /usr/local/bin/

# 创建服务文件
sudo vim /etc/systemd/system/rust-exporter.service
```

```ini
[Unit]
Description=Rust Exporter for Channel Health Monitoring
After=network.target

[Service]
Type=simple
User=exporter
Group=exporter
Environment="DATABASE_URL=postgres://user:password@host:5432/database"
ExecStart=/usr/local/bin/rust-exporter
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

#### 启动服务

```bash
# 创建用户
sudo useradd -r -s /bin/false exporter

# 重载 systemd
sudo systemctl daemon-reload

# 启动服务
sudo systemctl start rust-exporter

# 设置开机自启
sudo systemctl enable rust-exporter

# 查看状态
sudo systemctl status rust-exporter

# 查看日志
sudo journalctl -u rust-exporter -f
```

---

## 验证部署

### 1. 健康检查

```bash
# 检查服务是否运行
curl http://localhost:8001/health

# 预期输出: OK
```

### 2. 指标验证

```bash
# 获取所有指标
curl http://localhost:8001/metrics

# 检查特定指标
curl http://localhost:8001/metrics | grep channel_availability_percent
curl http://localhost:8001/metrics | grep channel_avg_cost_cny
curl http://localhost:8001/metrics | grep channel_cache_reuse_percent
```

预期看到：
```
# HELP channel_availability_percent Channel availability percentage
# TYPE channel_availability_percent gauge
channel_availability_percent{channel_group="aws"} 95.5
channel_availability_percent{channel_group="special"} 98.2

# HELP channel_avg_cost_cny_opus Average cost for Opus model
# TYPE channel_avg_cost_cny_opus gauge
channel_avg_cost_cny_opus{channel_group="aws"} 0.54
...
```

### 3. Prometheus 集成验证

```bash
# 访问 Prometheus UI
open http://localhost:9090

# 在 Status > Targets 中检查
# rust-exporter (localhost:8001) 应该是 UP 状态

# 在 Graph 中查询
channel_availability_percent
channel_avg_cost_cny_all
channel_cache_reuse_percent
```

### 4. Grafana 可视化验证

```bash
# 访问 Grafana
open http://localhost:3000
# 登录: admin / admin

# 检查数据源
# Configuration > Data Sources > prometheus 应该是绿色

# 查看 Dashboard
# Dashboards > 渠道健康状况 (Prometheus)
# 应该能看到所有指标的图表
```

### 5. 日志验证

```bash
# Docker 部署
docker compose logs rust-exporter | tail -50

# Systemd 部署
sudo journalctl -u rust-exporter -n 50

# Kubernetes 部署
kubectl logs -f deployment/rust-exporter
```

预期看到：
```
Starting Rust Exporter...
Database connection pool created
Registered 3 collectors
Starting HTTP server on 0.0.0.0:8001
Collected metrics from availability
Collected metrics from cache
Collected metrics from cost
```

---

## 监控和维护

### 性能监控

```bash
# Docker 资源使用
docker stats rust-exporter

# 预期:
# CPU: < 1%
# Memory: 8-15 MB
```

### 日志监控

```bash
# 实时查看日志
docker compose logs -f rust-exporter

# 查看错误日志
docker compose logs rust-exporter | grep -i error

# 查看最近 100 行
docker compose logs --tail=100 rust-exporter
```

### 定期检查

```bash
# 创建健康检查脚本
cat > /usr/local/bin/check-rust-exporter.sh << 'EOF'
#!/bin/bash

HEALTH_URL="http://localhost:8001/health"
METRICS_URL="http://localhost:8001/metrics"

# 检查健康状态
if ! curl -sf "$HEALTH_URL" > /dev/null; then
    echo "ERROR: Health check failed"
    exit 1
fi

# 检查指标数量
METRIC_COUNT=$(curl -s "$METRICS_URL" | grep -c "^channel_")
if [ "$METRIC_COUNT" -lt 10 ]; then
    echo "ERROR: Expected at least 10 metrics, got $METRIC_COUNT"
    exit 1
fi

echo "OK: Rust exporter is healthy"
exit 0
EOF

chmod +x /usr/local/bin/check-rust-exporter.sh

# 添加到 crontab (每 5 分钟检查一次)
echo "*/5 * * * * /usr/local/bin/check-rust-exporter.sh" | crontab -
```

### 更新部署

```bash
# 拉取最新代码
git pull origin main

# 重新构建
docker compose build rust-exporter

# 滚动更新 (零停机)
docker compose up -d rust-exporter

# 验证新版本
docker compose logs rust-exporter | head -20
```

---

## 故障排查

### 问题 1: 服务无法启动

**症状**: 容器启动后立即退出

**排查步骤**:
```bash
# 查看日志
docker compose logs rust-exporter

# 常见原因:
# 1. 数据库连接失败
# 2. 环境变量未设置
# 3. 端口被占用
```

**解决方案**:
```bash
# 检查数据库连接
psql "$DATABASE_URL" -c "SELECT 1"

# 检查环境变量
docker compose config | grep DATABASE_URL

# 检查端口占用
lsof -i :8001
lsof -i :8002
```

### 问题 2: 指标不更新

**症状**: /metrics 返回旧数据或空数据

**排查步骤**:
```bash
# 检查采集器是否运行
docker compose logs rust-exporter | grep "Collected metrics"

# 检查数据库查询
docker compose exec rust-exporter sh
# 在容器内测试 SQL
```

**解决方案**:
```bash
# 重启服务
docker compose restart rust-exporter

# 检查数据库权限
# 确保用户有 SELECT 权限
```

### 问题 3: Prometheus 无法抓取

**症状**: Prometheus Targets 显示 DOWN

**排查步骤**:
```bash
# 检查网络连接
docker compose exec prometheus wget -O- http://rust-exporter:8001/health

# 检查 Prometheus 配置
cat prometheus/prometheus.yml | grep rust-exporter
```

**解决方案**:
```bash
# 确保 prometheus.yml 中有配置
- job_name: 'rust-exporter'
  scrape_interval: 60s
  static_configs:
    - targets: ['rust-exporter:8001']

# 重启 Prometheus
docker compose restart prometheus
```

### 问题 4: 内存占用过高

**症状**: 内存使用超过 50MB

**排查步骤**:
```bash
# 检查内存使用
docker stats rust-exporter

# 检查是否有内存泄漏
docker compose logs rust-exporter | grep -i "panic\|error"
```

**解决方案**:
```bash
# 重启服务
docker compose restart rust-exporter

# 如果持续出现，检查代码或报告 issue
```

### 问题 5: CPU 使用率高

**症状**: CPU 使用超过 5%

**排查步骤**:
```bash
# 检查采集频率
docker compose logs rust-exporter | grep "Collected metrics" | tail -20

# 检查数据库查询性能
```

**解决方案**:
```bash
# 优化 SQL 查询
# 增加采集间隔 (修改 collector.rs 中的 interval())
# 添加数据库索引
```

---

## 回滚部署

### Docker 部署回滚

```bash
# 查看镜像历史
docker images rust-exporter

# 回滚到之前的版本
docker tag rust-exporter:previous rust-exporter:latest
docker compose up -d rust-exporter
```

### Git 回滚

```bash
# 查看提交历史
git log --oneline

# 回滚到特定提交
git checkout <commit-hash>
docker compose build rust-exporter
docker compose up -d rust-exporter
```

---

## 性能优化建议

### 1. 数据库优化

```sql
-- 添加索引以提升查询性能
CREATE INDEX idx_channel_request_log_created_at 
ON channel_request_log(created_at);

CREATE INDEX idx_channel_request_log_channel_code 
ON channel_request_log(channel_code);

CREATE INDEX idx_balance_transactions_created_at 
ON balance_transactions(created_at);
```

### 2. 连接池优化

编辑 `src/db.rs`:
```rust
let pool = PgPoolOptions::new()
    .max_connections(10)  // 增加最大连接数
    .min_connections(2)   // 增加最小连接数
    .acquire_timeout(Duration::from_secs(10))  // 增加超时
    .connect(database_url)
    .await?;
```

### 3. 采集间隔优化

编辑 `src/core/collector.rs`:
```rust
fn interval(&self) -> u64 {
    120  // 改为 120 秒，减少数据库压力
}
```

---

## 安全建议

### 1. 使用 Secret 管理

```bash
# 使用 Docker secrets
echo "postgres://user:pass@host:5432/db" | docker secret create db_url -

# 在 docker-compose.yml 中使用
secrets:
  - db_url
```

### 2. 限制网络访问

```yaml
# docker-compose.yml
services:
  rust-exporter:
    networks:
      - monitoring
    ports:
      - "127.0.0.1:8001:8001"  # 只监听本地
```

### 3. 使用只读数据库用户

```sql
-- 创建只读用户
CREATE USER exporter_readonly WITH PASSWORD 'secure_password';
GRANT CONNECT ON DATABASE your_db TO exporter_readonly;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO exporter_readonly;
```

---

## 总结

### 推荐部署方式

**开发环境**: 本地 cargo run  
**测试环境**: Docker Compose  
**生产环境**: Kubernetes 或 Docker Compose + Systemd

### 快速命令参考

```bash
# 启动
docker compose up -d rust-exporter

# 查看日志
docker compose logs -f rust-exporter

# 重启
docker compose restart rust-exporter

# 停止
docker compose stop rust-exporter

# 更新
git pull && docker compose build rust-exporter && docker compose up -d rust-exporter

# 健康检查
curl http://localhost:8001/health

# 查看指标
curl http://localhost:8001/metrics
```

---

**部署完成后，记得查看 [E2E_VALIDATION.md](./E2E_VALIDATION.md) 进行完整的验证！**
