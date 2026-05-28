#!/usr/bin/env bash
# install.sh - Installer for ruscut
# Tries GitHub Releases first (pre-built binary + SHA256 verify).
# Falls back to compiling from source via cargo if no release exists yet.
#
# Usage (from any directory):
#   ./scripts/install.sh [--version=X.Y.Z]

set -euo pipefail

# Always run from the project root (parent of this script's directory)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

REPO="rakaarwaky/ruscut"
INSTALL_DIR="${CARGO_HOME:-$HOME/.cargo}/bin"
mkdir -p "$INSTALL_DIR"

# Parse arguments
VERSION=""
for arg in "$@"; do
    case "$arg" in
        --help|-h)
            echo "Usage: $0 [--version=X.Y.Z]"
            echo ""
            echo "Installs ruscut (CLI) and ruscut-tui (TUI)."
            echo "Downloads from GitHub Releases, or builds from source as fallback."
            echo ""
            echo "Options:"
            echo "  --version=X, -v=X  Install specific version (e.g. --version=0.1.3)"
            echo "  --help, -h         Show this help"
            exit 0
            ;;
        --version=*|-v=*)
            VERSION="${arg#*=}"
            ;;
    esac
done

# ── Helper: build from source ────────────────────────────────────────────────
install_from_source() {
    echo ""
    echo "🔨 Building from source (this may take a few minutes)..."
    if ! command -v cargo >/dev/null 2>&1; then
        echo "❌ Neither a GitHub Release nor a local Rust toolchain is available."
        echo "   Install Rust: https://rustup.rs"
        exit 1
    fi
    cargo install --path "$PROJECT_ROOT" --force --locked
    VERSION_BUILT=$(grep -m 1 '^version = ' "$PROJECT_ROOT/Cargo.toml" | cut -d '"' -f 2)
    echo ""
    echo "✅ Built and installed ruscut v$VERSION_BUILT from source."
    echo "   ruscut     → $(command -v ruscut 2>/dev/null || echo "$INSTALL_DIR/ruscut")"
    echo "   ruscut-tui → $(command -v ruscut-tui 2>/dev/null || echo "$INSTALL_DIR/ruscut-tui")"
    exit 0
}

# ── Fetch GitHub Release data ────────────────────────────────────────────────
if ! command -v curl >/dev/null 2>&1; then
    echo "❌ 'curl' is required. Install it first."
    exit 1
fi

echo "🔍 Fetching release data from GitHub..."
API_BASE="https://api.github.com/repos/$REPO/releases"

RELEASE_JSON=""
if [ -n "$VERSION" ]; then
    RELEASE_JSON=$(curl -sSf "${API_BASE}/tags/v${VERSION}" 2>/dev/null \
        || curl -sSf "${API_BASE}/tags/${VERSION}" 2>/dev/null \
        || true)
else
    RELEASE_JSON=$(curl -sSf "${API_BASE}/latest" 2>/dev/null || true)
fi

# If no release found, fall back to building from source
if [ -z "$RELEASE_JSON" ] || echo "$RELEASE_JSON" | grep -q '"message"'; then
    echo "  ⚠️  No GitHub Release found for $REPO."
    echo "  Falling back to building from source..."
    install_from_source
fi

# ── Parse version and asset URLs ─────────────────────────────────────────────
if ! command -v jq >/dev/null 2>&1; then
    echo "  ⚠️  'jq' not found — cannot parse GitHub Release JSON."
    echo "  Falling back to building from source..."
    install_from_source
fi

if [ -z "$VERSION" ]; then
    VERSION=$(echo "$RELEASE_JSON" | jq -r '.tag_name' | sed 's/^v//')
    if [ -z "$VERSION" ] || [ "$VERSION" = "null" ]; then
        echo "  ⚠️  Could not parse version from release data."
        echo "  Falling back to building from source..."
        install_from_source
    fi
fi

echo "  ✅ Target version: v$VERSION"

# Detect OS and Architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
    x86_64)        ARCH="x86_64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    *)
        echo "  ⚠️  Unsupported architecture: $ARCH. Falling back to source build..."
        install_from_source
        ;;
esac
case "$OS" in
    linux)  OS="linux" ;;
    darwin) OS="macos" ;;
    *)
        echo "  ⚠️  Unsupported OS: $OS. Falling back to source build..."
        install_from_source
        ;;
esac

# Asset names match release.yml output (individual flat binaries, no tar.gz)
ASSET_CLI="ruscut-${OS}-${ARCH}"
ASSET_TUI="ruscut-tui-${OS}-${ARCH}"
CHECKSUM_ASSET="checksums.txt"

ASSET_CLI_URL=$(echo "$RELEASE_JSON" | jq -r --arg A "$ASSET_CLI" \
    '.assets[] | select(.name == $A) | .browser_download_url')
ASSET_TUI_URL=$(echo "$RELEASE_JSON" | jq -r --arg A "$ASSET_TUI" \
    '.assets[] | select(.name == $A) | .browser_download_url')
CHECKSUM_URL=$(echo "$RELEASE_JSON" | jq -r --arg A "$CHECKSUM_ASSET" \
    '.assets[] | select(.name == $A) | .browser_download_url')

if [ -z "$ASSET_CLI_URL" ] || [ "$ASSET_CLI_URL" = "null" ]; then
    echo "  ⚠️  Binary '$ASSET_CLI' not in release v$VERSION assets."
    echo "  Falling back to building from source..."
    install_from_source
fi

# ── Download binaries ─────────────────────────────────────────────────────────
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

echo ""
echo "⬇️  Downloading $ASSET_CLI..."
curl -sSfL "$ASSET_CLI_URL" -o "$TMP_DIR/$ASSET_CLI"

if [ -n "$ASSET_TUI_URL" ] && [ "$ASSET_TUI_URL" != "null" ]; then
    echo "⬇️  Downloading $ASSET_TUI..."
    curl -sSfL "$ASSET_TUI_URL" -o "$TMP_DIR/$ASSET_TUI"
else
    echo "  ⚠️  TUI binary not found in release assets. Skipping."
fi

# ── SHA256 verification ───────────────────────────────────────────────────────
if [ -n "$CHECKSUM_URL" ] && [ "$CHECKSUM_URL" != "null" ]; then
    echo ""
    echo "🔒 Verifying SHA256 checksums..."
    curl -sSfL "$CHECKSUM_URL" -o "$TMP_DIR/checksums.txt"
    cd "$TMP_DIR"
    if command -v sha256sum >/dev/null 2>&1; then
        grep "$ASSET_CLI" checksums.txt | sha256sum -c - >/dev/null && echo "  ✅ $ASSET_CLI OK"
        [ -f "$ASSET_TUI" ] && grep "$ASSET_TUI" checksums.txt | sha256sum -c - >/dev/null && echo "  ✅ $ASSET_TUI OK"
    elif command -v shasum >/dev/null 2>&1; then
        grep "$ASSET_CLI" checksums.txt | shasum -a 256 -c - >/dev/null && echo "  ✅ $ASSET_CLI OK"
        [ -f "$ASSET_TUI" ] && grep "$ASSET_TUI" checksums.txt | shasum -a 256 -c - >/dev/null && echo "  ✅ $ASSET_TUI OK"
    else
        echo "  ⚠️  No sha256sum/shasum found. Skipping verification."
    fi
    cd - >/dev/null
else
    echo "  ⚠️  No checksum file in release. Skipping verification."
fi

# ── Install ───────────────────────────────────────────────────────────────────
echo ""
echo "📦 Installing binaries to $INSTALL_DIR..."

install_binary() {
    local src="$TMP_DIR/$1"
    local dest="$INSTALL_DIR/$2"
    if [ -f "$src" ]; then
        cp "$src" "$dest"
        chmod +x "$dest"
        echo "  🚀 $2 → $INSTALL_DIR"
    fi
}

install_binary "$ASSET_CLI" "ruscut"
install_binary "$ASSET_TUI" "ruscut-tui"

echo ""
echo "✅ Installed ruscut v$VERSION"
echo "   ruscut     — CLI mode    (try: ruscut --help)"
echo "   ruscut-tui — TUI mode    (try: ruscut-tui)"
echo ""
echo "   Make sure $INSTALL_DIR is in your PATH."
