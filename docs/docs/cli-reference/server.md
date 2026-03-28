# ciab server

API server management.

## start

Start the CIAB API server.

```bash
ciab server start [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--config` | `config.toml` | Configuration file path |
| `--port` | 9090 | Override server port |
| `--host` | `0.0.0.0` | Override bind address |
| `--workers` | — | Number of worker threads |

```bash
# Start with defaults
ciab server start

# Custom port and config
ciab server start --config /etc/ciab/config.toml --port 9090
```

The server logs startup information and listens for HTTP requests. Use `Ctrl+C` to stop.
