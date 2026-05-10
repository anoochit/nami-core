# Observability Stack (OpenTelemetry + MLflow)

This directory contains the configuration for Nami's observability infrastructure, providing distributed tracing and experiment tracking.

## 🏗 Architecture

The tracing data flows from the application to MLflow through an OpenTelemetry Collector:

```text
Nami App (Rust)
  → OTLP gRPC (:4317)
      → OTel Collector
          → OTLP HTTP
              → MLflow Tracing UI (:5000)
```

## 🚀 Quick Start

1.  **Start the Docker Stack**:
    From this directory, run:
    ```bash
    docker compose up -d
    ```

2.  **Configure Nami**:
    Add the following to your root `.env` file to enable telemetry:
    ```text
    OTEL_COLLECTOR=http://localhost:4317
    ```

3.  **Access the Dashboard**:
    Open [http://localhost:5000](http://localhost:5000) in your browser to view traces and experiment logs.

## 🛠 Components

### MLflow
*   **Purpose**: Stores and visualizes traces.
*   **Storage**: 
    *   Metadata: `mlflow.db` (SQLite)
    *   Artifacts: `./mlruns/` directory
*   **Port**: `5000`

### OpenTelemetry Collector
*   **Purpose**: Receives, processes, and exports telemetry data.
*   **Receivers**: OTLP (gRPC on `4317`, HTTP on `4318`)
*   **Exporters**:
    *   `debug`: Prints detailed trace info to the container logs.
    *   `otlphttp/mlflow`: Forwards data to the MLflow backend.

## 🔍 Troubleshooting

### View Collector Logs
To see if traces are being received correctly:
```bash
docker compose logs -f otel-collector
```

### Rust Application Logs
Enable detailed logs for the OTLP exporter and networking layers:
```bash
# Windows (PowerShell)
$env:RUST_LOG="nami=info,adk_telemetry=debug,tonic=info,hyper=info"

# Linux/macOS
export RUST_LOG=nami=info,adk_telemetry=debug,tonic=info,hyper=info
```

### Environment Variables Reference
If manually configuring the OTLP exporter:
*   `OTEL_EXPORTER_OTLP_ENDPOINT`: `http://127.0.0.1:4317`
*   `OTEL_EXPORTER_OTLP_PROTOCOL`: `grpc`
*   `OTEL_EXPORTER_OTLP_INSECURE`: `true`
