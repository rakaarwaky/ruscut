#!/bin/bash
# ruscut - Installer
# Cross-platform installation script for Linux and macOS.
# Supports local build (default) or remote install from GitHub releases.

set -e

# Terminal colors
BOLD='\033[1m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
REPO="rakaarwaky/ruscut"

# Parse arguments
REMOTE_MODE=false
REMOTE_VERSION=""
for arg in "$@"; do
    case "$arg" in
        --remote|-r)
            REMOTE_MODE=true
            ;;
        --remote=*|-r=*)
            REMOTE_MODE=true
            REMOTE_VERSION="${arg#*=}"
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  (no args)        Install from local source (release build)"
            echo "  --remote, -r     Install latest release from GitHub"
            echo "  --remote=X, -r=X Install specific version from GitHub (e.g. -r=0.1.3)"
            echo "  --help, -h       Show this help"
            exit 0
            ;;
    esac
done

# Clear screen and show header
clear || true
echo -e "${BOLD}${BLUE}"
echo "=========================================================="
echo "    ____                         __                       "
echo "   / __ \__  ________________ __/ /_                      "
echo "  / /_/ / / / / ___/ ___/ __ \`/ __/                      "
echo " / _, _/ /_/ (__  ) /__/ /_/ / /_                        "
echo "/_/ |_|\__,_/____/\___/\__,_/\__/                        "
echo "                                                          "
echo "=========================================================="
echo "    AI-powered Background Remover CLI + TUI in Rust"
echo -e "${NC}"

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

# ============================================================
# REMOTE MODE: Download from GitHub Releases
# ============================================================
if [ "$REMOTE_MODE" = true ]; then
    echo -e "${BOLD}[1/3] Downloading from GitHub Releases...${NC}"

    # Detect version
    if [ -z "$REMOTE_VERSION" ]; then
        echo -e "  ${BLUE}Fetching latest release version...${NC}"
        REMOTE_VERSION=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | cut -d '"' -f 4 | sed 's/^v//')
        if [ -z "$REMOTE_VERSION" ]; then
            echo -e "${RED}ERROR: Could not fetch latest release version.${NC}"
            echo "  Check: https://github.com/$REPO/releases"
            exit 1
        fi
    fi
    echo -e "  ${GREEN}Target version: v$REMOTE_VERSION${NC}"

    # Detect OS and architecture
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)

    case "$OS" in
        linux)  OS_NAME="linux" ;;
        darwin) OS_NAME="macos" ;;
        *)      echo -e "${RED}ERROR: Unsupported OS: $OS${NC}"; exit 1 ;;
    esac

    case "$ARCH" in
        x86_64)  ARCH_NAME="x86_64" ;;
        aarch64|arm64) ARCH_NAME="aarch64" ;;
        *)       echo -e "${RED}ERROR: Unsupported architecture: $ARCH${NC}"; exit 1 ;;
    esac

    ASSET_NAME="ruscut-${OS_NAME}-${ARCH_NAME}.tar.gz"
    DOWNLOAD_URL="https://github.com/$REPO/releases/download/v${REMOTE_VERSION}/${ASSET_NAME}"

    echo -e "  ${BLUE}Downloading: $ASSET_NAME${NC}"
    TMP_DIR=$(mktemp -d)
    curl -sL "$DOWNLOAD_URL" -o "$TMP_DIR/$ASSET_NAME"

    if [ ! -f "$TMP_DIR/$ASSET_NAME" ] || [ ! -s "$TMP_DIR/$ASSET_NAME" ]; then
        echo -e "${RED}ERROR: Download failed. Asset may not exist for this platform.${NC}"
        echo "  URL: $DOWNLOAD_URL"
        echo "  Check: https://github.com/$REPO/releases/tag/v${REMOTE_VERSION}"
        rm -rf "$TMP_DIR"
        exit 1
    fi

    echo -e "  ${GREEN}Download complete.${NC}"
    echo -e "\n${BOLD}[2/3] Extracting binaries...${NC}"
    tar -xzf "$TMP_DIR/$ASSET_NAME" -C "$TMP_DIR"

    # Install binaries
    echo -e "\n${BOLD}[3/3] Installing binaries...${NC}"
    if [ -f "$TMP_DIR/ruscut" ]; then
        cp "$TMP_DIR/ruscut" "$INSTALL_DIR/ruscut"
        chmod +x "$INSTALL_DIR/ruscut"
        echo -e "  ${GREEN}Installed:  ruscut     -> $INSTALL_DIR/ruscut${NC}"
    fi
    if [ -f "$TMP_DIR/ruscut-tui" ]; then
        cp "$TMP_DIR/ruscut-tui" "$INSTALL_DIR/ruscut-tui"
        chmod +x "$INSTALL_DIR/ruscut-tui"
        echo -e "  ${GREEN}Installed:  ruscut-tui -> $INSTALL_DIR/ruscut-tui${NC}"
    fi

    rm -rf "$TMP_DIR"

    echo -e "\n=========================================================="
    echo -e "  ${GREEN}Remote installation complete! v$REMOTE_VERSION${NC}"
    echo -e "=========================================================="
    exit 0
fi

# ============================================================
# LOCAL MODE: Build from source
# ============================================================

# 1. Check for Rust and Cargo
echo -e "${BOLD}[1/4] Checking dependencies...${NC}"
if ! command -v cargo &>/dev/null; then
    echo -e "${YELLOW}Warning: Rust and Cargo are not installed!${NC}"
    echo "Rust is required to build and install ruscut."
    echo "Would you like to install Rustup now? (y/n)"
    read -r response
    if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
        echo -e "${BLUE}Running rustup installer...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    else
        echo -e "${RED}Installation cancelled. Please install Rust from https://rustup.rs/ and try again.${NC}"
        exit 1
    fi
else
    echo -e "  ${GREEN}Found Rust: $(cargo --version)${NC}"
fi

# Check for FFmpeg (required for video support)
if ! command -v ffmpeg &>/dev/null; then
    echo -e "  ${YELLOW}Warning: FFmpeg is not installed on your system!${NC}"
    echo "  FFmpeg is required to process video files (MP4, MOV, WebM, GIF, etc.)."
    echo "  (Note: Image processing will still work without FFmpeg.)"
    echo ""

    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if command -v apt-get &>/dev/null; then
            echo -n "  Would you like to install FFmpeg now via 'apt'? (y/n): "
            read -r ffmpeg_response
            if [[ "$ffmpeg_response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
                echo -e "${BLUE}Running: sudo apt-get update && sudo apt-get install -y ffmpeg...${NC}"
                sudo apt-get update && sudo apt-get install -y ffmpeg
            fi
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        if command -v brew &>/dev/null; then
            echo -n "  Would you like to install FFmpeg now via Homebrew? (y/n): "
            read -r ffmpeg_response
            if [[ "$ffmpeg_response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
                echo -e "${BLUE}Running: brew install ffmpeg...${NC}"
                brew install ffmpeg
            fi
        fi
    fi
else
    FFMPEG_VER=$(ffmpeg -version | head -n 1)
    echo -e "  ${GREEN}Found FFmpeg: $FFMPEG_VER${NC}"
fi

# 2. Build ALL binaries in release mode
echo -e "\n${BOLD}[2/4] Compiling all binaries in Release mode...${NC}"
echo -e "${BLUE}Compiling ruscut (CLI) and ruscut-tui (Interactive). Please wait...${NC}"
cargo build --release

# Verify both binaries were produced
if [ ! -f "target/release/ruscut" ]; then
    echo -e "${RED}ERROR: ruscut binary was not produced. Build may have failed.${NC}"
    exit 1
fi
if [ ! -f "target/release/ruscut-tui" ]; then
    echo -e "${RED}ERROR: ruscut-tui binary was not produced. Build may have failed.${NC}"
    exit 1
fi
echo -e "  ${GREEN}Build complete: ruscut and ruscut-tui produced.${NC}"

# 3. Install binaries
echo -e "\n${BOLD}[3/4] Installing binaries...${NC}"

cp target/release/ruscut "$INSTALL_DIR/ruscut"
echo -e "  ${GREEN}Installed:  ruscut     -> $INSTALL_DIR/ruscut${NC}"

cp target/release/ruscut-tui "$INSTALL_DIR/ruscut-tui"
echo -e "  ${GREEN}Installed:  ruscut-tui -> $INSTALL_DIR/ruscut-tui${NC}"

# 4. Verify PATH
echo -e "\n${BOLD}[4/4] Verifying installation PATH...${NC}"
CLI_OK=false
TUI_OK=false

if command -v ruscut &>/dev/null; then
    echo -e "  ${GREEN}OK: 'ruscut'     is ready in your PATH.${NC}"
    CLI_OK=true
fi
if command -v ruscut-tui &>/dev/null; then
    echo -e "  ${GREEN}OK: 'ruscut-tui' is ready in your PATH.${NC}"
    TUI_OK=true
fi

if [ "$CLI_OK" = false ] || [ "$TUI_OK" = false ]; then
    echo -e "\n  ${YELLOW}Notice: $INSTALL_DIR is not in your system PATH.${NC}"
    echo "  To run the binaries from anywhere, add it to your shell config."
    echo ""
    echo "  For Bash, add this line to ~/.bashrc:"
    echo -e "    ${BOLD}export PATH=\"$INSTALL_DIR:\$PATH\"${NC}"
    echo ""
    echo "  For Zsh, add this line to ~/.zshrc:"
    echo -e "    ${BOLD}export PATH=\"$INSTALL_DIR:\$PATH\"${NC}"
    echo ""
    echo "  Then reload your shell:"
    echo -e "    ${BOLD}source ~/.bashrc${NC}   (or source ~/.zshrc)"
fi

echo -e "\n=========================================================="
echo -e "  ${GREEN}Installation Complete! 2 binaries installed.${NC}"
echo -e "=========================================================="
echo ""
echo "CLI Mode (headless, for scripting and power users):"
echo "  ruscut input.jpg               # Remove background"
echo "  ruscut input.jpg output.png    # Custom output path"
echo "  ruscut --help                  # Show all options"
echo ""
echo "TUI Mode (interactive wizard, no commands to memorize):"
echo "  ruscut-tui                     # Launch interactive wizard"
echo ""
