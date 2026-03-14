# ciab agent

Interact with coding agents.

## chat

Chat with an agent in a sandbox.

```bash
ciab agent chat [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--sandbox-id` | Target sandbox (required) |
| `--session-id` | Existing session (creates new if omitted) |
| `--message`, `-m` | Message to send |
| `--interactive`, `-i` | Interactive mode (multi-turn) |
| `--stream`, `-s` | Stream output in real-time |

```bash
# Single message with streaming
ciab agent chat --sandbox-id <id> -m "Explain this codebase" -s

# Interactive session
ciab agent chat --sandbox-id <id> -i -s
```

## attach

Attach to an existing session's event stream.

```bash
ciab agent attach --session-id <sid>
```

Displays real-time events from the session.

## interrupt

Interrupt an agent's current processing.

```bash
ciab agent interrupt --session-id <sid>
```

## list providers

List available agent providers.

```bash
ciab agent providers
```
