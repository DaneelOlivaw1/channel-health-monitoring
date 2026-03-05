# Proxy Project Observatory

Complete monitoring stack for Prometheus + Grafana + Alertmanager with pre-configured dashboards and custom exporters.

## Features

- **Prometheus**: Metrics collection and storage
- **Grafana**: Visualization dashboards (System Overview, Application Metrics)
- **Alertmanager**: Alert routing and management
- **Node Exporter**: System-level metrics (CPU, memory, disk, network)
- **cAdvisor**: Container metrics
- **Custom Exporter**: Application-specific metrics example

## Quick Start

### Prerequisites

- Docker
- Docker Compose

### Start the Stack

```bash
docker-compose up -d
```

### Access Services

- **Grafana**: http://localhost:3000
  - Default credentials: `admin` / `admin`
  - Pre-loaded dashboards: System Overview, Application Metrics

- **Prometheus**: http://localhost:9090
  - Explore metrics and query data
  - Check alert rules status

- **Alertmanager**: http://localhost:9093
  - View active alerts
  - Configure alert routing

- **Node Exporter**: http://localhost:9100/metrics
- **cAdvisor**: http://localhost:8080
- **Custom Exporter**: http://localhost:8000/metrics

### Stop the Stack

```bash
docker-compose down
```

To remove volumes (data will be lost):

```bash
docker-compose down -v
```

## Configuration

### Environment Variables

Copy `.env.example` to `.env` and customize:

```bash
cp .env.example .env
```

Available variables:
- `GRAFANA_ADMIN_USER`: Grafana admin username (default: admin)
- `GRAFANA_ADMIN_PASSWORD`: Grafana admin password (default: admin)

### Prometheus Configuration

Edit `prometheus/prometheus.yml` to:
- Add scrape targets
- Adjust scrape intervals
- Configure service discovery

### Alert Rules

Alert rules are in `prometheus/alerts/rules.yml`:

- **Host Alerts**: CPU, memory, disk usage
- **Container Alerts**: Container health and resource usage
- **Application Alerts**: Error rates, response times

Edit thresholds and add custom rules as needed.

### Alertmanager Configuration

Configure alert routing in `alertmanager/alertmanager.yml`:

1. Update SMTP settings for email notifications
2. Configure webhook endpoints
3. Adjust routing rules

### Custom Exporter

The example exporter in `exporters/custom-exporter/` demonstrates:

- Gauge metrics (connections, CPU, memory)
- Counter metrics (request counts)
- Histogram metrics (request duration)

To customize:

1. Edit `exporters/custom-exporter/exporter.py`
2. Add your application-specific metrics
3. Rebuild: `docker-compose up -d --build custom-exporter`

## Dashboards

### System Overview

Monitors system resources:
- CPU usage (current + historical)
- Memory usage (current + historical)
- Disk space availability
- Network traffic
- Disk I/O
- Service status

### Application Metrics

Monitors application performance:
- Error rate
- Response time percentiles (p50, p95, p99)
- Request rate
- HTTP status code distribution
- Active connections

## Alert Rules

### Host Alerts

| Alert | Threshold | Duration | Severity |
|-------|-----------|----------|----------|
| HostDown | Service unavailable | 1m | critical |
| HighCPUUsage | >80% | 5m | warning |
| CriticalCPUUsage | >95% | 2m | critical |
| HighMemoryUsage | >80% | 5m | warning |
| CriticalMemoryUsage | >95% | 2m | critical |
| HighDiskUsage | <20% free | 5m | warning |
| CriticalDiskUsage | <10% free | 2m | critical |

### Container Alerts

| Alert | Condition | Duration | Severity |
|-------|-----------|----------|----------|
| ContainerDown | Container not running | 1m | warning |
| HighContainerCPU | >80% | 5m | warning |
| HighContainerMemory | >80% | 5m | warning |

### Application Alerts

| Alert | Threshold | Duration | Severity |
|-------|-----------|----------|----------|
| HighErrorRate | >5% | 5m | warning |
| SlowResponseTime | p95 >1s | 5m | warning |

## Extending the Stack

### Add New Scrape Target

Edit `prometheus/prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'my-service'
    static_configs:
      - targets: ['my-service:9090']
        labels:
          service: 'my-service'
```

### Add Custom Alert Rule

Create a new file in `prometheus/alerts/`:

```yaml
groups:
  - name: my_alerts
    interval: 30s
    rules:
      - alert: MyAlert
        expr: my_metric > 100
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "My custom alert"
```

### Import Additional Dashboards

1. Export dashboard JSON from Grafana UI
2. Save to `grafana/dashboards/`
3. Restart Grafana: `docker-compose restart grafana`

## Troubleshooting

### Check Service Logs

```bash
docker-compose logs prometheus
docker-compose logs grafana
docker-compose logs alertmanager
```

### Verify Targets

Open Prometheus UI (http://localhost:9090) → Status → Targets

All targets should show "UP" status.

### Reload Prometheus Configuration

```bash
curl -X POST http://localhost:9090/-/reload
```

### Dashboard Not Loading

1. Check Grafana logs: `docker-compose logs grafana`
2. Verify dashboard JSON syntax
3. Restart Grafana: `docker-compose restart grafana`

## Project Structure

```
proxy_project-observatory/
├── docker-compose.yml
├── .env.example
├── prometheus/
│   ├── prometheus.yml
│   └── alerts/
│       └── rules.yml
├── grafana/
│   ├── provisioning/
│   │   ├── datasources/
│   │   │   └── prometheus.yml
│   │   └── dashboards/
│   │       └── default.yml
│   └── dashboards/
│       ├── system-overview.json
│       └── application-metrics.json
├── alertmanager/
│   └── alertmanager.yml
└── exporters/
    └── custom-exporter/
        ├── Dockerfile
        ├── requirements.txt
        └── exporter.py
```

## License

MIT
