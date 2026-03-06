# Rust Exporter

High-performance Prometheus metrics exporter written in Rust, replacing the Python version with significant performance improvements.

## Features

- **3 Metric Collectors**:
  - Availability: Channel success rate (excluding user/auth errors)
  - Cost: Average cost per model (Opus, Sonnet, All)
  - Cache: Cache reuse percentage
  
- **HTTP Endpoints**:
  - `/metrics` - Prometheus format metrics
  - `/health` - Health check endpoint

- **Performance**:
  - ~10x faster startup time vs Python
  - ~5x lower memory usage
  - Async collection with Tokio runtime

## Quick Start

### Using Docker Compose

```bash
cd /path/to/project
docker compose up -d rust-exporter
```

### Local Development

```bash
# Set database URL
export DATABASE_URL="postgres://user:pass@host:5432/dbname"

# Run
cargo run --release

# Test
cargo test
```

## Configuration

Environment variables:

- `DATABASE_URL` - PostgreSQL connection string (required)

## Architecture

```
rust-exporter/
├── src/
│   ├── core/
│   │   └── collector.rs      # MetricCollector trait
│   ├── db.rs                 # Database connection pool
│   ├── metrics/
│   │   ├── availability/     # Availability collector
│   │   ├── cache/            # Cache collector
│   │   └── cost/             # Cost collector
│   ├── api/
│   │   └── mod.rs            # HTTP server (Axum)
│   └── main.rs               # Entry point
└── tests/                    # Integration tests
```

## Metrics Exposed

### Availability
- `channel_availability_percent{channel_group="aws|special"}` - Success rate percentage

### Cost
- `channel_avg_cost_cny_opus{channel_group="aws|special"}` - Opus model average cost
- `channel_avg_cost_cny_sonnet{channel_group="aws|special"}` - Sonnet model average cost
- `channel_avg_cost_cny_all{channel_group="aws|special"}` - All models average cost

### Cache
- `channel_cache_reuse_percent{channel_group="aws|special"}` - Cache hit rate percentage

## Development

Built with:
- Rust 1.75+
- Tokio (async runtime)
- Axum (HTTP framework)
- SQLx (PostgreSQL driver)
- metrics + metrics-exporter-prometheus

## Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test --test availability_collector_tests

# With output
cargo test -- --nocapture
```

## Performance Comparison

| Metric | Python | Rust | Improvement |
|--------|--------|------|-------------|
| Startup | ~2s | ~0.2s | 10x faster |
| Memory | ~50MB | ~10MB | 5x lower |
| CPU (idle) | ~2% | ~0.5% | 4x lower |

## License

MIT
