# 快速配置：推送到阿里云 Prometheus

## 1. 配置环境变量

编辑 `.env` 文件：

```bash
# 数据库连接
DATABASE_URL=postgres://dev_read_chunqiu:w7QcN8zp2VxT5Rb@pgm-6we2iwj50gul9ez8eo.pgsql.japan.rds.aliyuncs.com:5432/claude_code

# 阿里云 Prometheus Pushgateway（你的公网地址）
PUSHGATEWAY_URL=https://workspace-default-cms-1501153710058370-ap-northeast-1.ap-northeast-1.log.aliyuncs.com/prometheus/workspace-default-cms-1501153710058370-ap-northeast-1/aliyun-prom-rw-1c4ddb0a9425fcd0ecb589373fc6/api/v1/pushgateway

# Job 名称
PUSHGATEWAY_JOB=channel-health-exporter

# 阿里云 AccessKey（替换为你的实际值）
PUSHGATEWAY_USERNAME=LTAI5tXXXXXXXXXX
PUSHGATEWAY_PASSWORD=your_secret_key_here

# 推送间隔（秒）
PUSHGATEWAY_INTERVAL=60
```

## 2. 启动服务

```bash
# 构建
docker compose build rust-exporter

# 启动
docker compose up -d rust-exporter

# 查看日志
docker compose logs -f rust-exporter
```

## 3. 验证

### 查看日志（应该看到）：
```
Starting Rust Exporter...
Database connection pool created
Registered 3 collectors
Pushgateway is enabled, starting worker...
Starting Pushgateway worker: URL=https://workspace-default-cms-..., Job=channel-health-exporter, Interval=60s
Starting HTTP server on 0.0.0.0:8001
Collected metrics from availability
Successfully pushed metrics to Pushgateway
```

### 测试本地端点：
```bash
curl http://localhost:8001/health
curl http://localhost:8001/metrics | grep channel_
```

### 在阿里云控制台验证：
1. 登录 https://cloudmonitor.console.aliyun.com/
2. 左侧导航：Prometheus 监控 > 实例列表
3. 进入你的实例 > 大盘列表 > Explore
4. 查询：`channel_availability_percent`

## 4. 本地 Prometheus（可选，用于测试）

本地 Prometheus 仍然可以通过 http://localhost:8001/metrics 拉取数据，用于：
- 本地测试
- Grafana 看板
- 应急备份

## 故障排查

### 推送失败？
```bash
# 测试 AK/SK
curl -u 'your_ak:your_sk' \
  -H "Content-Type: text/plain; version=0.0.4; charset=utf-8" \
  --data-binary "test_metric 1.0" \
  "https://workspace-default-cms-1501153710058370-ap-northeast-1.ap-northeast-1.log.aliyuncs.com/prometheus/workspace-default-cms-1501153710058370-ap-northeast-1/aliyun-prom-rw-1c4ddb0a9425fcd0ecb589373fc6/api/v1/pushgateway/metrics/job/test"
```

### 401 错误？
- 检查 AK/SK 是否正确
- 确认 RAM 用户有 `AliyunPrometheusMetricWriteAccess` 权限

### 数据未显示？
- 等待 1-2 分钟（数据有延迟）
- 检查 Job 名称是否正确
- 查看阿里云控制台日志

## 完整文档

详细配置请查看：[PUSHGATEWAY_ALIYUN.md](./PUSHGATEWAY_ALIYUN.md)
