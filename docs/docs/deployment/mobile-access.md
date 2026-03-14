# Mobile Access

Chat with your coding agents from **any device** — iPhone, iPad, Android tablet, or any browser. CIAB's web UI works on mobile with full real-time streaming support.

![Gateway](../assets/screenshots/gateway.png)

## How It Works

CIAB serves a responsive web UI alongside the REST API. Any device on the same network can access it by navigating to the server URL. The Gateway page provides QR codes for instant mobile setup.

### Local Network (Same WiFi)

1. Start the CIAB server:

    ```bash
    ciab server start
    ```

2. Open the **Gateway** page in the desktop app or navigate to `http://localhost:9090/gateway`

3. Under **Local Access**, you'll see:
    - **mDNS URL**: `http://ciab.local.local:9090` — works on Apple devices automatically
    - **IP URL**: `http://<your-ip>:9090` — works on any device

4. Click the **QR code icon** next to either URL

5. **Scan the QR code** with your phone camera — it opens the CIAB web UI directly

### Remote Access (Anywhere)

For access outside your local network, enable a tunnel:

=== "Bore (free, no account needed)"

    ```toml
    # config.toml
    [gateway]
    enabled = true
    tunnel_provider = "bore"

    [gateway.bore]
    enabled = true
    auto_install = true  # downloads bore automatically
    ```

    ```bash
    ciab server start
    # Gateway will show: https://<random>.bore.pub
    ```

=== "Cloudflare Tunnel (free, reliable)"

    ```toml
    [gateway]
    enabled = true
    tunnel_provider = "cloudflare"

    [gateway.cloudflare]
    enabled = true
    auto_install = true
    ```

=== "ngrok (free tier available)"

    ```toml
    [gateway]
    enabled = true
    tunnel_provider = "ngrok"

    [gateway.ngrok]
    enabled = true
    authtoken = "your-ngrok-token"
    ```

Once a tunnel is active, the **Public Tunnels** section on the Gateway page shows the public URL with its own QR code. Share it or scan it from anywhere in the world.

## Mobile Chat Experience

The web UI is fully responsive. On mobile you get:

- **Full streaming chat** — text deltas arrive in real time, just like on desktop
- **Tool use visualization** — see Bash commands, file edits, and search results as they execute
- **Permission controls** — approve or deny tool calls from your phone
- **Session management** — create new sessions, switch between them
- **Message queue** — queue multiple messages while the agent is working

## Security

### Token-Scoped Access

Generate access tokens with specific permissions:

```bash
# Create a read-only token
ciab gateway token create --name "mobile-read" --scope read

# Create a full-access token
ciab gateway token create --name "mobile-full" --scope admin
```

Tokens can be revoked at any time from the Gateway page.

### LAN Discovery

CIAB advertises itself on the local network via mDNS (Bonjour). Devices on the same WiFi can find it at `http://ciab.local.local:9090` without knowing the IP address.

To disable LAN discovery:

```toml
[gateway.lan]
enabled = false
```

## Tips

- **Bookmark the URL** on your phone's home screen for app-like access
- **Use the mDNS URL** (`ciab.local.local`) on Apple devices — it updates automatically if your IP changes
- **Enable a tunnel** if you want to chat with agents while away from home
- The web UI uses the same SSE streaming as the desktop app — no polling, no delays
