#!/usr/bin/env bash
# release.sh - Full Release Pipeline (2026 Standard)
# Bumps version, generates changelog, commits, tags, and pushes.
# GitHub Actions then builds binaries and uploads them to the release.
#
# Prerequisites: cargo install cargo-release
# Optional:      cargo install git-cliff (for changelog generation)
#
# Usage (from any directory):
#   ./scripts/release.sh [major|minor|patch|X.Y.Z] [--skip-publish]

set -euo pipefail

# Always run from the project root (parent of this script's directory)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

BUMP_TYPE="${1:-patch}"

# Validate prerequisites
if ! command -v cargo-release >/dev/null 2>&1; then
    echo "❌ 'cargo-release' not installed."
    echo "   Install: cargo install cargo-release"
    exit 1
fi

# Validate input
if [[ ! "$BUMP_TYPE" =~ ^(major|minor|patch|alpha|beta|rc|[0-9]+\.[0-9]+\.[0-9]+.*)$ ]]; then
    echo "❌ Invalid bump format: $BUMP_TYPE"
    echo "   Usage: $0 <major|minor|patch|version>"
    exit 1
fi

# Parse flags
SKIP_PUSH=false
for arg in "$@"; do
    if [ "$arg" = "--skip-push" ]; then
        SKIP_PUSH=true
    fi
done

CURRENT=$(grep -m 1 '^version = ' Cargo.toml | cut -d '"' -f 2)
echo "🚀 Starting release pipeline: $CURRENT → ($BUMP_TYPE)"
echo "   Working directory: $PROJECT_ROOT"
echo ""

echo "📝 [1/3] Generating changelog from conventional commits..."
if command -v git-cliff >/dev/null 2>&1; then
    git-cliff --output CHANGELOG.md
    git add CHANGELOG.md
    git commit -m "docs: update changelog for release" 2>/dev/null || echo "  No changelog changes to commit."
else
    echo "  ⚠️  git-cliff not installed. Skipping changelog."
    echo "  Install: cargo install git-cliff"
fi

echo ""
echo "🏷️  [2/3] Bumping version and tagging release ($BUMP_TYPE)..."
# cargo-release: bumps Cargo.toml, commits, creates a git tag, and pushes.
# --no-publish: ruscut distributes via GitHub Releases, not crates.io.
if [ "$SKIP_PUSH" = true ]; then
    if ! cargo release "$BUMP_TYPE" \
        --execute \
        --no-publish \
        --no-push \
        --no-confirm; then
        echo ""
        echo "❌ cargo-release failed. Run 'git status' and fix uncommitted changes."
        exit 1
    fi
else
    if ! cargo release "$BUMP_TYPE" \
        --execute \
        --no-publish \
        --no-confirm; then
        echo ""
        echo "❌ cargo-release failed. Common causes:"
        echo "   - Uncommitted changes in working tree (run: git status)"
        echo "   - No git remote configured (run: git remote -v)"
        echo "   - Push access denied"
        exit 1
    fi
fi

NEW_VERSION=$(grep -m 1 '^version = ' Cargo.toml | cut -d '"' -f 2)
echo ""
echo "✅ [3/3] Release pipeline complete! v$NEW_VERSION"
if [ "$SKIP_PUSH" = true ]; then
    echo "   Bump committed locally. Push manually when ready."
else
    echo "   Tag v$NEW_VERSION pushed → GitHub Actions will build binaries."
    echo "   Monitor: https://github.com/rakaarwaky/ruscut/actions"
fi
