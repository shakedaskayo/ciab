# Docker Deployment

## Docker Compose

```yaml
version: '3.8'

services:
  ciab:
    build: .
    ports:
      - "9090:9090"
    environment:
      - CIAB_ENCRYPTION_KEY=${CIAB_ENCRYPTION_KEY}
      - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}
    volumes:
      - ./config.toml:/etc/ciab/config.toml
      - ciab-data:/var/lib/ciab
    command: ciab server start --config /etc/ciab/config.toml
    depends_on:
      - opensandbox

  opensandbox:
    image: opensandbox/opensandbox:latest
    ports:
      - "8000:8000"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock

volumes:
  ciab-data:
```

## Building the Image

```dockerfile
FROM rust:1.75-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin ciab

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates sqlite3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/ciab /usr/local/bin/ciab
ENTRYPOINT ["ciab"]
```

## Running

```bash
# Generate encryption key
export CIAB_ENCRYPTION_KEY=$(openssl rand -base64 32)

# Start services
docker compose up -d

# Verify
curl http://localhost:9090/health
```
