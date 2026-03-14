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

# ── Auth header for private repo support ─────────────────────────────
AUTH_HEADER=""
if [ -n "${GITHUB_TOKEN:-}" ]; then
  AUTH_HEADER="Authorization: token ${GITHUB_TOKEN}"
fi

curl_auth() {
  if [ -n "$AUTH_HEADER" ]; then
    curl -H "$AUTH_HEADER" "$@"
  else
    curl "$@"
  fi
}

# ── Parse args ────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case "$1" in
    --version|-v) VERSION="$2"; shift 2 ;;
    --dir|-d)     INSTALL_DIR="$2"; shift 2 ;;
    --help|-h)
      echo "Usage: install.sh [--version VERSION] [--dir INSTALL_DIR]"
      echo ""
      echo "Environment:"
      echo "  GITHUB_TOKEN        GitHub token (required for private repos)"
      echo "  CIAB_INSTALL_DIR    Install directory (default: /usr/local/bin)"
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
  latest=$(curl_auth -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' | head -1 | cut -d'"' -f4)

  if [ -z "$latest" ]; then
    echo "Error: Could not determine latest version." >&2
    echo "  - For private repos, set GITHUB_TOKEN" >&2
    echo "  - Or specify a version with --version" >&2
    exit 1
  fi

  echo "$latest"
}

# ── Download release asset (handles private repo redirect) ───────────
download_asset() {
  local url="$1"
  local dest="$2"

  if [ -n "${GITHUB_TOKEN:-}" ]; then
    # For private repos, use the GitHub API to get the asset download URL
    # GitHub releases redirect to S3, but private repos need Accept header
    curl_auth -fSL --progress-bar \
      -H "Accept: application/octet-stream" \
      "$url" -o "$dest"
  else
    curl -fSL --progress-bar "$url" -o "$dest"
  fi
}

# ── Verify checksum ──────────────────────────────────────────────────
verify_checksum() {
  local archive="$1"
  local checksum_file="$2"

  echo "Verifying checksum..."
  local expected actual
  expected=$(awk '{print $1}' "$checksum_file")

  if command -v sha256sum &>/dev/null; then
    actual=$(sha256sum "$archive" | awk '{print $1}')
  elif command -v shasum &>/dev/null; then
    actual=$(shasum -a 256 "$archive" | awk '{print $1}')
  else
    echo "  Warning: No sha256sum or shasum found, skipping checksum verification"
    return 0
  fi

  if [ "$expected" = "$actual" ]; then
    echo "  Checksum OK"
  else
    echo "Error: Checksum mismatch!" >&2
    echo "  Expected: $expected" >&2
    echo "  Got:      $actual" >&2
    exit 1
  fi
}

# ── Main ──────────────────────────────────────────────────────────────
TMP_DIR=""
cleanup() { rm -rf "${TMP_DIR:-}"; }
trap cleanup EXIT

main() {
  local platform version artifact_name download_url

  platform=$(detect_platform)
  version=$(resolve_version)
  artifact_name="${platform}.tar.gz"
  download_url="https://github.com/${REPO}/releases/download/${version}/${artifact_name}"

  echo "Installing CIAB ${version} (${platform})..."
  echo "  From: ${download_url}"
  echo "  To:   ${INSTALL_DIR}/ciab"
  echo ""

  TMP_DIR=$(mktemp -d)

  # Download
  if ! download_asset "$download_url" "${TMP_DIR}/${artifact_name}"; then
    echo ""
    echo "Error: Failed to download ${download_url}"
    echo "Check that the version '${version}' exists at:"
    echo "  https://github.com/${REPO}/releases"
    if [ -z "${GITHUB_TOKEN:-}" ]; then
      echo ""
      echo "For private repos, set GITHUB_TOKEN:"
      echo "  export GITHUB_TOKEN=ghp_..."
    fi
    exit 1
  fi

  # Verify checksum if available
  local sha_url="${download_url}.sha256"
  if curl_auth -fsSL "$sha_url" -o "${TMP_DIR}/checksum.sha256" 2>/dev/null; then
    verify_checksum "${TMP_DIR}/${artifact_name}" "${TMP_DIR}/checksum.sha256"
  fi

  # Extract
  tar xzf "${TMP_DIR}/${artifact_name}" -C "$TMP_DIR"

  # Verify extracted binary runs
  if ! "${TMP_DIR}/ciab" --version &>/dev/null; then
    echo "Error: Extracted binary failed to run. Wrong platform?" >&2
    exit 1
  fi

  # Install
  mkdir -p "$INSTALL_DIR" 2>/dev/null || true
  if [ -w "$INSTALL_DIR" ]; then
    mv "${TMP_DIR}/ciab" "${INSTALL_DIR}/ciab"
  else
    echo "Need sudo to install to ${INSTALL_DIR}"
    sudo mv "${TMP_DIR}/ciab" "${INSTALL_DIR}/ciab"
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
