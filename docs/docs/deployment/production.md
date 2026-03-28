# Production Deployment

Recommendations for running CIAB in production.

## TLS Termination

CIAB does not handle TLS directly. Use a reverse proxy:

```nginx
server {
    listen 443 ssl;
    server_name ciab.example.com;

    ssl_certificate /etc/ssl/certs/ciab.pem;
    ssl_certificate_key /etc/ssl/private/ciab.key;

    location / {
        proxy_pass http://127.0.0.1:9090;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # SSE streams need long timeouts
    location ~ /stream$ {
        proxy_pass http://127.0.0.1:9090;
        proxy_buffering off;
        proxy_cache off;
        proxy_read_timeout 3600s;
        proxy_set_header Connection '';
        proxy_http_version 1.1;
        chunked_transfer_encoding off;
    }
}
```

## Authentication

Enable API key authentication:

```toml
[security]
api_keys = ["your-secret-api-key-1", "your-secret-api-key-2"]
```

All requests must include `Authorization: Bearer <api-key>`.

## CORS

Restrict CORS origins in production:

```toml
[server]
cors_origins = ["https://ciab.example.com", "tauri://localhost"]
```

## Security Hardening

### Drop Capabilities

```toml
[security]
drop_capabilities = ["NET_RAW", "SYS_ADMIN", "SYS_PTRACE"]
```

### Resource Limits

Set default resource limits per provider to prevent runaway containers.

### Network Policies

Restrict container network access:

```json
{
  "network": {
    "enabled": true,
    "allowed_hosts": ["api.anthropic.com", "api.openai.com"],
    "dns_servers": ["8.8.8.8"]
  }
}
```

## Logging

Configure structured logging for observability:

```toml
[logging]
level = "info"
format = "json"
```

Pipe logs to your preferred aggregator (e.g., Datadog, Grafana Loki).

## Scaling

- **Horizontal**: Run multiple CIAB instances behind a load balancer. SSE streams are per-connection, so use sticky sessions.
- **Vertical**: Increase `server.workers` for more concurrent requests.
- **Database**: SQLite is suitable for single-instance deployments. For multi-instance, consider migrating to PostgreSQL.
