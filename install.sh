#!/bin/bash
# ruscut - Easy Installer
# Cross-platform installation script for Linux and macOS.
# Compiles and installs BOTH ruscut (CLI) and ruscut-tui (Interactive TUI).

set -e

# Terminal colors
BOLD='\033[1m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

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
        # Source Cargo environment
        source "$HOME/.cargo/env"
    else
        echo -e "${RED}Installation cancelled. Please install Rust from https://rustup.rs/ and try again.${NC}"
        exit 1
    fi
else
    echo -e "  ${GREEN}Found: $(cargo --version)${NC}"
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

# 3. Determine installation path
echo -e "\n${BOLD}[3/4] Installing binaries...${NC}"
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

# Install CLI binary
cp target/release/ruscut "$INSTALL_DIR/ruscut"
echo -e "  ${GREEN}Installed:  ruscut     -> $INSTALL_DIR/ruscut${NC}"

# Install TUI binary
cp target/release/ruscut-tui "$INSTALL_DIR/ruscut-tui"
echo -e "  ${GREEN}Installed:  ruscut-tui -> $INSTALL_DIR/ruscut-tui${NC}"

# 4. Check if the binaries are in PATH
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
echo "  ruscut --fp16 input.jpg        # Use FP16 model"
echo "  ruscut --full input.jpg        # Use Full precision model"
echo "  ruscut --help                  # Show all options"
echo ""
echo "TUI Mode (interactive wizard, no commands to memorize):"
echo "  ruscut-tui                     # Launch interactive wizard"
echo ""
