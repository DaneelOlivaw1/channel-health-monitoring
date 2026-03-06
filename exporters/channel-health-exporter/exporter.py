import time
import os
import json
import psycopg2
from psycopg2 import pool
from prometheus_client import start_http_server
from datetime import datetime
from http.server import HTTPServer, BaseHTTPRequestHandler
from threading import Thread
from metrics import (
    channel_availability,
    channel_cache_reuse_rate,
    channel_avg_cost_opus,
    channel_avg_cost_sonnet,
    channel_avg_cost_all,
    get_metrics_config,
)

EXPORTER_PORT = int(os.getenv("EXPORTER_PORT", "8001"))
SCRAPE_INTERVAL = int(os.getenv("SCRAPE_INTERVAL", "60"))

DB_HOST = os.getenv("DB_HOST", "localhost")
DB_PORT = int(os.getenv("DB_PORT", "5432"))
DB_NAME = os.getenv("DB_NAME", "claude_code")
DB_USER = os.getenv("DB_USER", "dev_read_chunqiu")
DB_PASSWORD = os.getenv("DB_PASSWORD", "")


class ChannelHealthExporter:
    def __init__(self):
        self.connection_pool = psycopg2.pool.SimpleConnectionPool(
            1,
            5,
            host=DB_HOST,
            port=DB_PORT,
            database=DB_NAME,
            user=DB_USER,
            password=DB_PASSWORD,
        )
        print(f"Connected to PostgreSQL at {DB_HOST}:{DB_PORT}/{DB_NAME}")

    def get_connection(self):
        return self.connection_pool.getconn()

    def return_connection(self, conn):
        self.connection_pool.putconn(conn)

    def collect_availability_metrics(self):
        """采集可用性指标"""
        conn = self.get_connection()
        try:
            cursor = conn.cursor()

            # 聚合指标（最近3小时）
            query = """
            SELECT
                CASE WHEN channel_code='aws' THEN 'aws' ELSE 'special' END as grp,
                ROUND(
                    SUM(CASE WHEN status='success' THEN 1 ELSE 0 END)::numeric * 100.0
                    / NULLIF(SUM(CASE WHEN
                        NOT (
                            response_code IN (400,404,413,429)
                            OR (response_code = -1 AND error_message LIKE '%unconfigured%')
                            OR response_code IN (401,403)
                            OR (response_code = 500 AND error_message LIKE '%credit balance%')
                        )
                        OR status='success' THEN 1 ELSE 0 END), 0)
                , 1) as availability
            FROM channel_request_log
            WHERE created_at >= NOW() - INTERVAL '3 hours'
                AND channel_code IN ('claude_laohu_max','claude_steven','claude_steven_az','claude_laohu_official','aws')
            GROUP BY grp
            """

            cursor.execute(query)
            for row in cursor.fetchall():
                channel_group, availability = row
                if availability is not None:
                    channel_availability.labels(channel_group=channel_group).set(
                        availability
                    )

            cursor.close()
        finally:
            self.return_connection(conn)

    def collect_cache_metrics(self):
        """采集缓存复用率指标"""
        conn = self.get_connection()
        try:
            cursor = conn.cursor()

            query = """
            SELECT
                CASE WHEN channel_used='aws' THEN 'aws' ELSE 'special' END as grp,
                ROUND(
                    SUM(cache_read_tokens)::numeric * 100.0
                    / NULLIF(SUM(cache_read_tokens) + SUM(cache_creation_tokens), 0)
                , 1) as cache_reuse
            FROM balance_transactions
            WHERE created_at >= NOW() - INTERVAL '3 hours'
                AND type='consume' AND transaction_status='completed'
                AND channel_used IN ('claude_laohu_max','claude_steven','claude_steven_az','claude_laohu_official','aws')
            GROUP BY grp
            """

            cursor.execute(query)
            for row in cursor.fetchall():
                channel_group, cache_reuse = row
                if cache_reuse is not None:
                    channel_cache_reuse_rate.labels(channel_group=channel_group).set(
                        cache_reuse
                    )

            cursor.close()
        finally:
            self.return_connection(conn)

    def collect_cost_metrics(self):
        """采集成本指标"""
        conn = self.get_connection()
        try:
            cursor = conn.cursor()

            query = """
            SELECT
                CASE WHEN channel_used='aws' THEN 'aws' ELSE 'special' END as grp,
                ROUND(AVG(CASE WHEN model_name LIKE '%opus%' THEN final_price_cny END)::numeric, 2) as opus_price,
                ROUND(AVG(CASE WHEN model_name LIKE '%sonnet%' THEN final_price_cny END)::numeric, 2) as sonnet_price
            FROM balance_transactions
            WHERE created_at >= NOW() - INTERVAL '3 hours'
                AND type='consume' AND transaction_status='completed'
                AND channel_used IN ('claude_laohu_max','claude_steven','claude_steven_az','claude_laohu_official','aws')
            GROUP BY grp
            """

            cursor.execute(query)
            for row in cursor.fetchall():
                channel_group, opus_price, sonnet_price = row
                if opus_price is not None:
                    channel_avg_cost_opus.labels(channel_group=channel_group).set(
                        opus_price
                    )
                if sonnet_price is not None:
                    channel_avg_cost_sonnet.labels(channel_group=channel_group).set(
                        sonnet_price
                    )

            cursor.close()
        finally:
            self.return_connection(conn)

    def collect_all_cost_metrics(self):
        conn = self.get_connection()
        try:
            cursor = conn.cursor()

            query = """
            SELECT
                CASE WHEN channel_used='aws' THEN 'aws' ELSE 'special' END as grp,
                ROUND(AVG(final_price_cny)::numeric, 2) as avg_price
            FROM balance_transactions
            WHERE created_at >= NOW() - INTERVAL '3 hours'
                AND type='consume' AND transaction_status='completed'
                AND channel_used IN ('claude_laohu_max','claude_steven','claude_steven_az','claude_laohu_official','aws')
            GROUP BY grp
            """

            cursor.execute(query)
            for row in cursor.fetchall():
                channel_group, avg_price = row
                if avg_price is not None:
                    channel_avg_cost_all.labels(channel_group=channel_group).set(
                        avg_price
                    )

            cursor.close()
        finally:
            self.return_connection(conn)

    def collect_all_metrics(self):
        try:
            print(f"[{datetime.now()}] Collecting metrics...")
            self.collect_availability_metrics()
            self.collect_cache_metrics()
            self.collect_cost_metrics()
            self.collect_all_cost_metrics()
            print(f"[{datetime.now()}] Metrics collected successfully")
        except Exception as e:
            print(f"[{datetime.now()}] Error collecting metrics: {e}")

    def run(self):
        """主循环"""
        while True:
            self.collect_all_metrics()
            time.sleep(SCRAPE_INTERVAL)


class MetadataHandler(BaseHTTPRequestHandler):
    """HTTP handler for serving metrics metadata"""

    def do_GET(self):
        if self.path == "/metrics-metadata":
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Access-Control-Allow-Origin", "*")
            self.end_headers()

            # Get metrics config directly from YAML
            metrics_list = get_metrics_config()

            self.wfile.write(
                json.dumps(metrics_list, ensure_ascii=False, indent=2).encode("utf-8")
            )
        else:
            self.send_response(404)
            self.end_headers()

    def log_message(self, format, *args):
        # Suppress default logging
        pass


if __name__ == "__main__":
    # Start Prometheus metrics server
    start_http_server(EXPORTER_PORT)
    print(f"Channel Health Exporter started on port {EXPORTER_PORT}")
    print(f"Metrics endpoint: http://localhost:{EXPORTER_PORT}/metrics")
    print(f"Metadata endpoint: http://localhost:{EXPORTER_PORT}/metrics-metadata")
    print(f"Scrape interval: {SCRAPE_INTERVAL}s")

    # Start metadata HTTP server on the same port (using prometheus_client's server)
    # We need to add a custom handler to the existing server
    # Since prometheus_client doesn't support custom endpoints easily,
    # we'll start a separate server on port 8002 for metadata
    METADATA_PORT = EXPORTER_PORT + 1
    metadata_server = HTTPServer(("0.0.0.0", METADATA_PORT), MetadataHandler)
    metadata_thread = Thread(target=metadata_server.serve_forever, daemon=True)
    metadata_thread.start()
    print(f"Metadata API started on port {METADATA_PORT}")
    print(f"Access at: http://localhost:{METADATA_PORT}/metrics-metadata")

    exporter = ChannelHealthExporter()
    exporter.run()
