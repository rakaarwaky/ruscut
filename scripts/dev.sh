#!/usr/bin/env bash
# dev.sh - Local Development Build & Install
# Runs quality gates (fmt + clippy) then installs both binaries locally.
# For production install from GitHub Releases, use install.sh.
#
# Usage (from any directory):
#   ./scripts/dev.sh

set -euo pipefail

# Always run from the project root (parent of this script's directory)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

echo "🔍 [1/3] Running Quality Gates..."
echo "   Working directory: $PROJECT_ROOT"
echo ""

# Ensure code is formatted
if ! cargo fmt --all --check >/dev/null 2>&1; then
    echo "❌ Code is not formatted. Run 'cargo fmt' to fix."
    echo "   Unformatted files:"
    cargo fmt --all --check 2>&1 | grep '^Diff' | sed 's/^/   /'
    exit 1
fi
echo "  ✅ Formatting OK"

# Ensure no clippy warnings (treat warnings as errors — matches CI)
if ! cargo clippy --all-targets --all-features -- -D warnings 2>&1; then
    echo "❌ Clippy found warnings or errors. Fix them before installing."
    exit 1
fi
echo "  ✅ Clippy OK"

echo ""
echo "🛠️  [2/3] Building & Installing local binaries..."
# cargo install --path:
#   - Builds in --release mode automatically
#   - Discovers all [[bin]] targets (ruscut and ruscut-tui)
#   - Installs to ~/.cargo/bin with correct permissions
#   - --force  : overwrites existing binary
#   - --locked : respects Cargo.lock for reproducible builds
cargo install --path . --force --locked

VERSION=$(grep -m 1 '^version = ' Cargo.toml | cut -d '"' -f 2)
echo ""
echo "🎉 [3/3] Installation complete! v$VERSION"
echo "   ruscut     → $(command -v ruscut 2>/dev/null || echo '~/.cargo/bin/ruscut')"
echo "   ruscut-tui → $(command -v ruscut-tui 2>/dev/null || echo '~/.cargo/bin/ruscut-tui')"
echo ""
echo "   Try: ruscut --help"
