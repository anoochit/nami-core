# Garage Quickstart

The latest stable Garage release is v2.3.0. To get started, you will need a garage.toml config file alongside your docker-compose.yml. Place both files in the same folder and follow these steps

**1. Generate secure secrets** (replace the placeholders in `garage.toml`):
```bash
openssl rand -hex 32   # → rpc_secret
openssl rand -base64 32  # → admin_token
openssl rand -base64 32  # → metrics_token
```

**2. Start everything:**
```bash
docker compose up -d
```

**3. Access points:**

| Service | URL | Notes |
|---|---|---|
| Garage S3 API | `http://localhost:3900` | Use with any S3 SDK |
| Garage Web UI | `http://localhost:3909` | Browse buckets & keys |
| Adminer | `http://localhost:8080` | Postgres GUI |

The Web UI (`garage-webui`) is the equivalent of MinIO's console — you can create buckets, manage access keys, and browse objects from it.