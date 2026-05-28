#!/usr/bin/env bash
# install.sh - Secure Installer (2026 Standard)
# Install ruscut from GitHub Releases with SHA256 verification.
# For local development build, use dev.sh instead.

set -euo pipefail

REPO="rakaarwaky/ruscut"
INSTALL_DIR="${CARGO_HOME:-$HOME/.cargo}/bin"
mkdir -p "$INSTALL_DIR"

# Parse arguments
VERSION=""
for arg in "$@"; do
    case "$arg" in
        --help|-h)
            echo "Usage: $0 [--version=X]"
            echo ""
            echo "Install ruscut from GitHub Releases."
            echo "For local build: ./scripts/dev.sh"
            echo ""
            echo "Options:"
            echo "  --version=X, -v=X  Install specific version (e.g. -v=0.1.3)"
            echo "  --help, -h         Show this help"
            exit 0
            ;;
        --version=*|-v=*)
            VERSION="${arg#*=}"
            ;;
    esac
done

# Prerequisite check
if ! command -v curl >/dev/null 2>&1; then
    echo "❌ 'curl' is required. Install it first."
    exit 1
fi
if ! command -v jq >/dev/null 2>&1; then
    echo "❌ 'jq' is required. Install: sudo apt install jq"
    exit 1
fi

echo "🔍 Fetching release data from GitHub..."
API_URL="https://api.github.com/repos/$REPO/releases"

if [ -n "$VERSION" ]; then
    RELEASE_JSON=$(curl -sSf "${API_URL}/tags/v${VERSION}" 2>/dev/null \
        || curl -sSf "${API_URL}/tags/${VERSION}" 2>/dev/null \
        || true)
else
    RELEASE_JSON=$(curl -sSf "${API_URL}/latest" 2>/dev/null || true)
fi

if [ -z "$RELEASE_JSON" ] || echo "$RELEASE_JSON" | grep -q '"message".*"Not Found"'; then
    echo "❌ Could not fetch release data."
    if [ -n "$VERSION" ]; then
        echo "   Release v$VERSION not found at: https://github.com/$REPO/releases"
    else
        echo "   No releases published yet at: https://github.com/$REPO/releases"
        echo "   Use the local build instead: ./scripts/dev.sh"
    fi
    exit 1
fi

if [ -z "$VERSION" ]; then
    VERSION=$(echo "$RELEASE_JSON" | jq -r '.tag_name' | sed 's/^v//')
    if [ -z "$VERSION" ] || [ "$VERSION" = "null" ]; then
        echo "❌ Failed to parse version from release data."
        exit 1
    fi
fi

echo "  ✅ Target version: v$VERSION"

# Detect OS and Architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
    x86_64)  ARCH="x86_64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    *) echo "❌ Unsupported architecture: $ARCH"; exit 1 ;;
esac
case "$OS" in
    linux)  OS="linux" ;;
    darwin) OS="macos" ;;
    *) echo "❌ Unsupported OS: $OS"; exit 1 ;;
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
    echo "❌ Asset '$ASSET_CLI' not found in release v$VERSION"
    echo "   Check: https://github.com/$REPO/releases/tag/v$VERSION"
    exit 1
fi

# Download with trap cleanup
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

echo ""
echo "⬇️  Downloading $ASSET_CLI..."
curl -sSfL "$ASSET_CLI_URL" -o "$TMP_DIR/$ASSET_CLI"

echo "⬇️  Downloading $ASSET_TUI..."
if [ -n "$ASSET_TUI_URL" ] && [ "$ASSET_TUI_URL" != "null" ]; then
    curl -sSfL "$ASSET_TUI_URL" -o "$TMP_DIR/$ASSET_TUI"
else
    echo "  ⚠️  TUI binary not found in release. Skipping."
fi

# Supply Chain Security: SHA256 verification
if [ -n "$CHECKSUM_URL" ] && [ "$CHECKSUM_URL" != "null" ]; then
    echo ""
    echo "🔒 Verifying SHA256 checksums..."
    curl -sSfL "$CHECKSUM_URL" -o "$TMP_DIR/checksums.txt"

    cd "$TMP_DIR"
    if command -v sha256sum >/dev/null 2>&1; then
        grep "$ASSET_CLI" checksums.txt | sha256sum -c - >/dev/null && echo "  ✅ $ASSET_CLI checksum OK"
        if [ -f "$ASSET_TUI" ]; then
            grep "$ASSET_TUI" checksums.txt | sha256sum -c - >/dev/null && echo "  ✅ $ASSET_TUI checksum OK"
        fi
    elif command -v shasum >/dev/null 2>&1; then
        grep "$ASSET_CLI" checksums.txt | shasum -a 256 -c - >/dev/null && echo "  ✅ $ASSET_CLI checksum OK"
        if [ -f "$ASSET_TUI" ]; then
            grep "$ASSET_TUI" checksums.txt | shasum -a 256 -c - >/dev/null && echo "  ✅ $ASSET_TUI checksum OK"
        fi
    else
        echo "  ⚠️  No sha256sum/shasum found. Verification skipped."
    fi
    cd - >/dev/null
else
    echo "  ⚠️  Checksum file not found in release. Continuing without verification."
fi

echo ""
echo "📦 Installing binaries..."

install_binary() {
    local src="$TMP_DIR/$1"
    local dest_name="$2"
    if [ -f "$src" ]; then
        cp "$src" "$INSTALL_DIR/$dest_name"
        chmod +x "$INSTALL_DIR/$dest_name"
        echo "  🚀 Installed: $dest_name → $INSTALL_DIR"
    fi
}

install_binary "$ASSET_CLI" "ruscut"
install_binary "$ASSET_TUI" "ruscut-tui"

echo ""
echo "✅ Successfully installed ruscut v$VERSION!"
echo "   ruscut     - CLI mode (try: ruscut --help)"
echo "   ruscut-tui - TUI interactive mode (try: ruscut-tui)"
echo ""
echo "   Make sure $INSTALL_DIR is in your PATH."
