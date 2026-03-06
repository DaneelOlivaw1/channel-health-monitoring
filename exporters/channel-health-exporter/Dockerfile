FROM python:3.11-slim

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY exporter.py metrics.py metrics.yaml .

EXPOSE 8001 8002

CMD ["python", "exporter.py"]
