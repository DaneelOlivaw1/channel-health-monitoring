import time
import random
from prometheus_client import start_http_server, Gauge, Counter, Histogram
import os

EXPORTER_PORT = int(os.getenv("EXPORTER_PORT", 8000))

active_connections = Gauge(
    "custom_app_active_connections", "Number of active connections"
)
request_counter = Counter(
    "http_requests_total", "Total HTTP requests", ["method", "path", "status"]
)
request_duration = Histogram(
    "http_request_duration_seconds", "HTTP request duration", ["method", "path"]
)

cpu_usage = Gauge("custom_app_cpu_usage_percent", "Application CPU usage percentage")
memory_usage = Gauge(
    "custom_app_memory_usage_bytes", "Application memory usage in bytes"
)
processing_queue_size = Gauge("custom_app_queue_size", "Size of processing queue")


def simulate_metrics():
    while True:
        active_connections.set(random.randint(10, 100))

        methods = ["GET", "POST", "PUT", "DELETE"]
        paths = ["/api/users", "/api/products", "/api/orders", "/api/health"]
        statuses = ["200", "201", "400", "404", "500"]

        method = random.choice(methods)
        path = random.choice(paths)
        status = random.choices(statuses, weights=[70, 10, 10, 5, 5])[0]

        request_counter.labels(method=method, path=path, status=status).inc()

        duration = random.uniform(0.01, 2.0)
        request_duration.labels(method=method, path=path).observe(duration)

        cpu_usage.set(random.uniform(10, 90))
        memory_usage.set(random.randint(100_000_000, 500_000_000))
        processing_queue_size.set(random.randint(0, 50))

        time.sleep(1)


if __name__ == "__main__":
    start_http_server(EXPORTER_PORT)
    print(f"Custom Exporter started on port {EXPORTER_PORT}")
    simulate_metrics()
