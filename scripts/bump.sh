#!/usr/bin/env bash
# bump.sh - Local Version Bump (No Push, No Publish, No Tag)
# Uses cargo-release for safe TOML parsing.
# Prerequisite: cargo install cargo-release
#
# Usage (from any directory):
#   ./scripts/bump.sh [major|minor|patch|X.Y.Z]

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
    echo ""
    echo "Examples:"
    echo "  $0 patch    # 0.1.3 -> 0.1.4"
    echo "  $0 minor    # 0.1.3 -> 0.2.0"
    echo "  $0 major    # 0.1.3 -> 1.0.0"
    echo "  $0 0.5.0    # explicit version"
    exit 1
fi

CURRENT=$(grep -m 1 '^version = ' Cargo.toml | cut -d '"' -f 2)
echo "📝 Bumping version ($BUMP_TYPE) from $CURRENT..."
echo "   Working directory: $PROJECT_ROOT"

# cargo-release modifies Cargo.toml & Cargo.lock using native TOML parser (100% safe)
# Flags below prevent push, tag, or publish — local-only version bump
if ! cargo release "$BUMP_TYPE" \
    --execute \
    --no-publish \
    --no-push \
    --no-tag \
    --no-confirm \
    --allow-branch "*"; then
    echo ""
    echo "❌ cargo-release failed. Common causes:"
    echo "   - Uncommitted changes in working tree (run: git status)"
    echo "   - Invalid version bump type"
    echo "   Current version: $CURRENT"
    exit 1
fi

NEW_VERSION=$(grep -m 1 '^version = ' Cargo.toml | cut -d '"' -f 2)
echo ""
echo "✅ Version bumped to $NEW_VERSION (committed locally)"
echo "   No push, no tag, no publish."
echo "   Use ./scripts/release.sh when ready to publish a release."
