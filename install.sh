#!/usr/bin/env bash
# CIAB installer — https://github.com/shakedaskayo/ciab
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/shakedaskayo/ciab/main/install.sh | bash
#   curl -fsSL https://raw.githubusercontent.com/shakedaskayo/ciab/main/install.sh | bash -s -- --version v0.1.0

set -euo pipefail

REPO="shakedaskayo/ciab"
INSTALL_DIR="${CIAB_INSTALL_DIR:-/usr/local/bin}"
VERSION=""

# ── Parse args ────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case "$1" in
    --version|-v) VERSION="$2"; shift 2 ;;
    --dir|-d)     INSTALL_DIR="$2"; shift 2 ;;
    --help|-h)
      echo "Usage: install.sh [--version VERSION] [--dir INSTALL_DIR]"
      exit 0
      ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

# ── Detect platform ──────────────────────────────────────────────────
detect_platform() {
  local os arch

  case "$(uname -s)" in
    Darwin) os="darwin" ;;
    Linux)  os="linux" ;;
    *)      echo "Error: Unsupported OS: $(uname -s)"; exit 1 ;;
  esac

  case "$(uname -m)" in
    x86_64|amd64)  arch="x64" ;;
    aarch64|arm64) arch="arm64" ;;
    *)             echo "Error: Unsupported architecture: $(uname -m)"; exit 1 ;;
  esac

  echo "ciab-${os}-${arch}"
}

# ── Resolve version ──────────────────────────────────────────────────
resolve_version() {
  if [ -n "$VERSION" ]; then
    echo "$VERSION"
    return
  fi

  local latest
  latest=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' | head -1 | cut -d'"' -f4)

  if [ -z "$latest" ]; then
    echo "Error: Could not determine latest version. Specify one with --version." >&2
    exit 1
  fi

  echo "$latest"
}

# ── Main ──────────────────────────────────────────────────────────────
main() {
  local platform version artifact_name download_url tmp_dir

  platform=$(detect_platform)
  version=$(resolve_version)
  artifact_name="${platform}.tar.gz"
  download_url="https://github.com/${REPO}/releases/download/${version}/${artifact_name}"

  echo "Installing CIAB ${version} (${platform})..."
  echo "  From: ${download_url}"
  echo "  To:   ${INSTALL_DIR}/ciab"
  echo ""

  tmp_dir=$(mktemp -d)
  trap 'rm -rf "$tmp_dir"' EXIT

  # Download
  if ! curl -fSL --progress-bar "$download_url" -o "${tmp_dir}/${artifact_name}"; then
    echo ""
    echo "Error: Failed to download ${download_url}"
    echo "Check that the version '${version}' exists at:"
    echo "  https://github.com/${REPO}/releases"
    exit 1
  fi

  # Verify checksum if available
  local sha_url="${download_url}.sha256"
  if curl -fsSL "$sha_url" -o "${tmp_dir}/checksum.sha256" 2>/dev/null; then
    echo "Verifying checksum..."
    cd "$tmp_dir"
    if command -v sha256sum &>/dev/null; then
      sha256sum -c checksum.sha256
    elif command -v shasum &>/dev/null; then
      shasum -a 256 -c checksum.sha256
    fi
    cd - >/dev/null
  fi

  # Extract
  tar xzf "${tmp_dir}/${artifact_name}" -C "$tmp_dir"

  # Install
  if [ -w "$INSTALL_DIR" ]; then
    mv "${tmp_dir}/ciab" "${INSTALL_DIR}/ciab"
  else
    echo "Need sudo to install to ${INSTALL_DIR}"
    sudo mv "${tmp_dir}/ciab" "${INSTALL_DIR}/ciab"
  fi

  chmod +x "${INSTALL_DIR}/ciab"

  echo ""
  echo "CIAB ${version} installed successfully!"
  echo ""
  echo "Get started:"
  echo "  ciab --help"
  echo "  ciab config init"
  echo "  ciab server start"
}

main
