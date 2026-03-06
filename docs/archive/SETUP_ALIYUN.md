# 配置 Rust Exporter 推送到阿里云 Prometheus

## 步骤 1: 复制配置文件

```bash
cd /Users/daneel/project/proxy_project-observatory
cp .env.example .env
```

## 步骤 2: 编辑 .env 文件

```bash
vim .env
```

添加以下配置：

```bash
# 阿里云 Prometheus Pushgateway 配置
PUSHGATEWAY_URL=https://workspace-default-cms-1501153710058370-ap-northeast-1.ap-northeast-1.log.aliyuncs.com/prometheus/workspace-default-cms-1501153710058370-ap-northeast-1/aliyun-prom-rw-1c4ddb0a9425fcd0ecb589373fc6/api/v1/pushgateway
PUSHGATEWAY_JOB=channel-health-exporter
PUSHGATEWAY_USERNAME=你的AccessKey_ID
PUSHGATEWAY_PASSWORD=你的AccessKey_Secret
PUSHGATEWAY_INTERVAL=60
```

## 步骤 3: 启动服务

```bash
# 构建
docker compose build rust-exporter

# 启动
docker compose up -d rust-exporter

# 查看日志
docker compose logs -f rust-exporter
```

## 步骤 4: 验证

### 查看日志
```bash
docker compose logs rust-exporter | grep -i pushgateway
```

应该看到：
```
Pushgateway is enabled, starting worker...
Starting Pushgateway worker: URL=https://workspace-default-cms-..., Job=channel-health-exporter, Interval=60s
Successfully pushed metrics to Pushgateway
```

### 测试本地端点
```bash
curl http://localhost:8001/health
curl http://localhost:8001/metrics | grep channel_
```

### 在阿里云控制台验证
1. 登录 https://cloudmonitor.console.aliyun.com/
2. Prometheus 监控 > 实例列表
3. 进入你的实例 > 大盘列表 > Explore
4. 查询：`channel_availability_percent`

## 注意事项

1. **RAM 权限**：确保 AccessKey 有 `AliyunPrometheusMetricWriteAccess` 权限
2. **网络连通性**：确保服务器能访问阿里云公网地址
3. **本地 Prometheus**：仍然可以通过 http://localhost:8001/metrics 拉取数据

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
- 确认 RAM 用户有权限

### 数据未显示？
- 等待 1-2 分钟
- 检查 Job 名称
- 查看阿里云控制台日志

## 完整文档

详细配置请查看：
- [QUICKSTART_ALIYUN.md](./docs/rust-exporter/QUICKSTART_ALIYUN.md)
- [PUSHGATEWAY_ALIYUN.md](./docs/rust-exporter/PUSHGATEWAY_ALIYUN.md)
