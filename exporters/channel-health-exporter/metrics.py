"""
Centralized Metrics Registry - Dynamically loaded from YAML

This module reads metrics.yaml and dynamically creates Prometheus metrics.
"""

import yaml
from prometheus_client import Gauge, Counter, Histogram, Summary
from pathlib import Path


def load_metrics_config():
    """Load metrics configuration from YAML file"""
    config_path = Path(__file__).parent / "metrics.yaml"
    with open(config_path, "r", encoding="utf-8") as f:
        return yaml.safe_load(f)


def create_metric(metric_config):
    """Create a Prometheus metric based on configuration"""
    metric_type = metric_config["type"]
    name = metric_config["name"]
    description = metric_config["description"]
    labels = metric_config.get("labels", [])

    if metric_type == "Gauge":
        return Gauge(name, description, labels)
    elif metric_type == "Counter":
        return Counter(name, description, labels)
    elif metric_type == "Histogram":
        return Histogram(name, description, labels)
    elif metric_type == "Summary":
        return Summary(name, description, labels)
    else:
        raise ValueError(f"Unknown metric type: {metric_type}")


# Load configuration and create metrics dynamically
_config = load_metrics_config()
_metrics = {}

for metric_config in _config["metrics"]:
    metric_name = metric_config["name"]
    _metrics[metric_name] = create_metric(metric_config)


def get_metric(name):
    """Get a metric by name"""
    return _metrics.get(name)


def get_all_metrics():
    """Get all metrics"""
    return _metrics


def get_metrics_config():
    """Get the raw metrics configuration"""
    return _config["metrics"]


# Export commonly used metrics for backward compatibility
channel_availability = get_metric("channel_availability_percent")
channel_cache_reuse_rate = get_metric("channel_cache_reuse_percent")
channel_avg_cost_opus = get_metric("channel_avg_cost_cny_opus")
channel_avg_cost_sonnet = get_metric("channel_avg_cost_cny_sonnet")
channel_avg_cost_all = get_metric("channel_avg_cost_cny_all")
