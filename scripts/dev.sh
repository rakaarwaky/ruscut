#!/bin/bash
# dev.sh - Quick debug build + install (for development)
# Faster than release build. Use during active development.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Terminal colors
BOLD='\033[1m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

# Determine install directory
LOCAL_BIN="$HOME/.local/bin"
CARGO_BIN="$HOME/.cargo/bin"
INSTALL_DIR=""

if [ -d "$CARGO_BIN" ]; then
    INSTALL_DIR="$CARGO_BIN"
elif [ -d "$LOCAL_BIN" ]; then
    INSTALL_DIR="$LOCAL_BIN"
else
    mkdir -p "$LOCAL_BIN"
    INSTALL_DIR="$LOCAL_BIN"
fi

echo -e "${BOLD}${BLUE}[dev] Quick debug build + install${NC}"
echo ""

# Build debug mode (fast)
echo -e "${BLUE}Building debug binaries (fast, no optimization)...${NC}"
cd "$PROJECT_ROOT"
cargo build

if [ ! -f "target/debug/ruscut" ]; then
    echo -e "${RED}ERROR: ruscut binary was not produced.${NC}"
    exit 1
fi
if [ ! -f "target/debug/ruscut-tui" ]; then
    echo -e "${RED}ERROR: ruscut-tui binary was not produced.${NC}"
    exit 1
fi

# Install
cp target/debug/ruscut "$INSTALL_DIR/ruscut"
cp target/debug/ruscut-tui "$INSTALL_DIR/ruscut-tui"
chmod +x "$INSTALL_DIR/ruscut" "$INSTALL_DIR/ruscut-tui"

VERSION=$(grep -m 1 '^version = ' "$PROJECT_ROOT/Cargo.toml" | cut -d '"' -f 2)
echo -e "${GREEN}Installed debug build v$VERSION -> $INSTALL_DIR${NC}"
echo -e "  ruscut     (debug)"
echo -e "  ruscut-tui (debug)"
