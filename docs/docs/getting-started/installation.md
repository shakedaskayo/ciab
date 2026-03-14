# Installation

## Quick Install (recommended)

Install the latest release with a single command:

=== "macOS / Linux"

    ```bash
    curl -fsSL https://raw.githubusercontent.com/shakedaskayo/ciab/main/install.sh | bash
    ```

=== "Specific version"

    ```bash
    curl -fsSL https://raw.githubusercontent.com/shakedaskayo/ciab/main/install.sh | bash -s -- --version v0.1.0
    ```

=== "Custom directory"

    ```bash
    curl -fsSL https://raw.githubusercontent.com/shakedaskayo/ciab/main/install.sh | bash -s -- --dir ~/.local/bin
    ```

This downloads the pre-built binary for your platform and places it in `/usr/local/bin`.

## Download from GitHub Releases

Pre-built binaries are available for every tagged release:

| Platform | Architecture | Artifact |
|----------|-------------|----------|
| macOS | Apple Silicon (M1+) | `ciab-darwin-arm64.tar.gz` |
| macOS | Intel | `ciab-darwin-x64.tar.gz` |
| Linux | x86_64 | `ciab-linux-x64.tar.gz` |
| Linux | ARM64 | `ciab-linux-arm64.tar.gz` |

Download from [GitHub Releases](https://github.com/shakedaskayo/ciab/releases/latest), extract, and move to your PATH:

```bash
tar xzf ciab-darwin-arm64.tar.gz
sudo mv ciab /usr/local/bin/
```

## Desktop App

Download the macOS desktop app (`.dmg`) from [GitHub Releases](https://github.com/shakedaskayo/ciab/releases/latest).

## Build from Source

Requires [Rust](https://rustup.rs) (stable, latest).

```bash
git clone https://github.com/shakedaskayo/ciab.git
cd ciab
cargo build --release
```

The `ciab` binary will be at `target/release/ciab`.

```bash
# Install to PATH
sudo cp target/release/ciab /usr/local/bin/
```

## Verify Installation

```bash
ciab --version
ciab --help
```

## Initialize Configuration

Generate a default config file:

```bash
ciab config init
```

This creates `config.toml` in the current directory. See [Configuration](../configuration/index.md) for details on all settings.
