# Binaries

Install additional command-line tools and binaries in the sandbox environment.

## Configuration

```toml
[[workspace.binaries]]
name = "ripgrep"
method = "apt"
version = "14.1.0"
```

## Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | string | Yes | — | Package/binary name |
| `method` | string | No | `"apt"` | Install method |
| `version` | string | No | latest | Specific version |
| `install_command` | string | No | — | Override install command entirely |

## Install Methods

| Method | Command Generated | Example |
|--------|------------------|---------|
| `apt` | `apt-get install -y {name}` | System packages |
| `cargo` | `cargo install {name}` | Rust tools |
| `npm` | `npm install -g {name}` | Node.js tools |
| `pip` | `pip install {name}` | Python tools |
| `url` | Download from URL to `/usr/local/bin/` | Pre-built binaries |
| `custom` | Uses `install_command` field | Anything else |

## Examples

```toml
# System packages
[[workspace.binaries]]
name = "ripgrep"
method = "apt"

[[workspace.binaries]]
name = "jq"
method = "apt"

# Rust tools
[[workspace.binaries]]
name = "fd-find"
method = "cargo"

# Node.js tools
[[workspace.binaries]]
name = "typescript"
method = "npm"
version = "5.4"

# Custom install
[[workspace.binaries]]
name = "custom-tool"
method = "custom"
install_command = "curl -fsSL https://install.example.com | bash"
```
