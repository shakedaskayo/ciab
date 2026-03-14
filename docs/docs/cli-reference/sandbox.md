# ciab sandbox

Manage sandbox lifecycle.

## create

Create a new sandbox.

```bash
ciab sandbox create [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--provider` | `claude-code` | Agent provider |
| `--name` | — | Sandbox name |
| `--image` | — | Custom container image |
| `--cpu` | — | CPU core limit |
| `--memory` | — | Memory limit (MB) |
| `--disk` | — | Disk limit (MB) |
| `--env`, `-e` | — | Environment variable (repeatable) |
| `--git-repo` | — | Git repo URL to clone (repeatable) |
| `--credential` | — | Credential ID to inject (repeatable) |
| `--timeout` | 300 | Provisioning timeout (seconds) |

```bash
ciab sandbox create --provider claude-code --name my-project \
  -e ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY \
  --git-repo https://github.com/user/repo.git
```

## list

List all sandboxes.

```bash
ciab sandbox list [--state <state>] [--provider <provider>]
```

## get

Get sandbox details.

```bash
ciab sandbox get <sandbox-id>
```

## delete

Delete a sandbox (irreversible).

```bash
ciab sandbox delete <sandbox-id>
```

## start / stop / pause / resume

Control sandbox state.

```bash
ciab sandbox start <sandbox-id>
ciab sandbox stop <sandbox-id>
ciab sandbox pause <sandbox-id>
ciab sandbox resume <sandbox-id>
```

## stats

Show resource statistics.

```bash
ciab sandbox stats <sandbox-id>
```

## logs

View sandbox logs.

```bash
ciab sandbox logs <sandbox-id> [--follow] [--tail <n>]
```

## exec

Execute a command in the sandbox.

```bash
ciab sandbox exec <sandbox-id> [--workdir <path>] [--timeout <secs>] -- <command...>
```

```bash
ciab sandbox exec abc123 -- cargo build --release
ciab sandbox exec abc123 --workdir /workspace/src -- ls -la
```
