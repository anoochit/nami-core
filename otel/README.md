```
export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317
export OTEL_EXPORTER_OTLP_PROTOCOL=grpc
export OTEL_EXPORTER_OTLP_INSECURE=true
```

```
export RUST_LOG=my_cli=info,hyper=info,h2=info,tonic=info
```

`docker-compose.yml`

```yaml id="gkv5j8"
services:
  mlflow:
    image: ghcr.io/mlflow/mlflow:latest
    container_name: mlflow
    restart: unless-stopped

    ports:
      - "5000:5000"

    command: >
      mlflow server
      --host 0.0.0.0
      --port 5000
      --backend-store-uri sqlite:///mlflow.db
      --allowed-hosts "*"

    volumes:
      - ./mlruns:/mlflow/mlruns
      - ./mlflow.db:/mlflow/mlflow.db

    working_dir: /mlflow

  otel-collector:
    image: otel/opentelemetry-collector-contrib:latest
    container_name: otel-collector
    restart: unless-stopped

    depends_on:
      - mlflow

    ports:
      - "4317:4317"
      - "4318:4318"

    command:
      - --config=/etc/otelcol-contrib/config.yaml

    volumes:
      - ./otel-collector.yaml:/etc/otelcol-contrib/config.yaml
```

`otel-collector.yaml`

```yaml id="ahz6oe"
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch:

exporters:
  debug:
    verbosity: detailed

  otlphttp/mlflow:
    endpoint: http://mlflow:5000
    headers:
      x-mlflow-experiment-id: "1"

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch]
      exporters:
        - debug
        - otlphttp/mlflow
```

Rust:

```rust id="d5i7q4"
use adk_telemetry::init_with_otlp;

init_with_otlp(
    "my-agent",
    "http://localhost:4317",
)?;
```

Start stack:

```bash id="jlwm6x"
docker compose up -d
```

View logs:

```bash id="o2gm4d"
docker compose logs -f otel-collector
```

MLflow UI:

```text id="o91khz"
http://localhost:5000
```

Expected flow:

```text id="dgzntg"
Rust App
  → OTLP gRPC (:4317)
      → OTel Collector
          → OTLP HTTP
              → MLflow /v1/traces
```
