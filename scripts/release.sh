#!/bin/bash

# Script to bump version, commit, push, and trigger GitHub Release
# Usage: ./scripts/release.sh <major|minor|patch|x.y.z> ["optional custom commit message"]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CARGO_TOML="$PROJECT_ROOT/Cargo.toml"

if [ -z "$1" ]; then
    echo "Usage: $0 <major|minor|patch|x.y.z> [\"optional commit message\"]"
    exit 1
fi

BUMP_TYPE=$1
CUSTOM_MSG=$2

if [ ! -f "$CARGO_TOML" ]; then
    echo "Error: Cargo.toml not found at $CARGO_TOML"
    exit 1
fi

# Extract current version
CURRENT_VERSION=$(grep -m 1 '^version = ' "$CARGO_TOML" | cut -d '"' -f 2)

if [ -z "$CURRENT_VERSION" ]; then
    echo "Error: Could not determine current version from Cargo.toml"
    exit 1
fi

IFS='.' read -r major minor patch <<< "$CURRENT_VERSION"

# Determine new version
if [ "$BUMP_TYPE" == "major" ]; then
    major=$((major + 1))
    minor=0
    patch=0
    NEW_VERSION="${major}.${minor}.${patch}"
elif [ "$BUMP_TYPE" == "minor" ]; then
    minor=$((minor + 1))
    patch=0
    NEW_VERSION="${major}.${minor}.${patch}"
elif [ "$BUMP_TYPE" == "patch" ]; then
    patch=$((patch + 1))
    NEW_VERSION="${major}.${minor}.${patch}"
else
    # Check if the provided version matches x.y.z format
    if [[ ! "$BUMP_TYPE" =~ ^[0-9]+\.[0-9]+\.[0-9]+.*$ ]]; then
        echo "Error: Invalid version format '$BUMP_TYPE'. Must be 'major', 'minor', 'patch', or a valid semantic version string."
        exit 1
    fi
    NEW_VERSION="$BUMP_TYPE"
fi

echo "Bumping version from $CURRENT_VERSION to $NEW_VERSION..."

# Update version in Cargo.toml (only the first occurrence under [package])
awk -v old="version = \"$CURRENT_VERSION\"" -v new="version = \"$NEW_VERSION\"" '
    !done && $0 == old {
        print new
        done = 1
        next
    }
    {print}
' "$CARGO_TOML" > "${CARGO_TOML}.tmp" && mv "${CARGO_TOML}.tmp" "$CARGO_TOML"

# Update Cargo.lock to reflect the new version
echo "Updating Cargo.lock..."
cd "$PROJECT_ROOT"
cargo check > /dev/null 2>&1 || true

# Commit and Push
echo "Staging files..."
git add .

if [ -z "$CUSTOM_MSG" ]; then
    COMMIT_MSG="chore: release v$NEW_VERSION"
else
    COMMIT_MSG="$CUSTOM_MSG"
fi

echo "Committing with message: '$COMMIT_MSG'..."
git commit -m "$COMMIT_MSG"

# Create tag locally
echo "Creating tag v$NEW_VERSION..."
git tag "v$NEW_VERSION"

echo "Publishing to crates.io..."
# We run cargo publish before pushing to GitHub.
# If it fails, the script stops and won't push the tag to GitHub.
cargo publish || {
    echo "❌ Failed to publish to crates.io. Did you forget to 'cargo login'?"
    echo "You can manually fix the issue, run 'cargo publish', and then manually push:"
    echo "  git push && git push origin v$NEW_VERSION"
    exit 1
}

echo "Pushing code to repository..."
git push

echo "Pushing tag to repository to trigger GitHub Actions release..."
git push origin "v$NEW_VERSION"

echo ""
echo "✅ Successfully bumped version to $NEW_VERSION, published to crates.io, committed, pushed, and tagged!"
echo "🚀 GitHub Actions (.github/workflows/release.yml) will now automatically build and publish the release."
