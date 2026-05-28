#!/bin/bash

# bump.sh - Version bump + commit only (no push, no release)
# Usage: ./scripts/bump.sh <major|minor|patch|x.y.z> ["optional commit message"]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CARGO_TOML="$PROJECT_ROOT/Cargo.toml"

BUMP_TYPE="$1"
CUSTOM_MSG="$2"

if [ -z "$BUMP_TYPE" ]; then
    echo "Usage: $0 <major|minor|patch|x.y.z> [\"commit message\"]"
    echo ""
    echo "Examples:"
    echo "  $0 patch \"fix: description\"     # 0.1.3 -> 0.1.4"
    echo "  $0 minor \"feat: description\"     # 0.1.3 -> 0.2.0"
    echo "  $0 major \"chore: breaking\"       # 0.1.3 -> 1.0.0"
    echo "  $0 0.5.0 \"custom version\"        # explicit version"
    exit 1
fi

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
    if [[ ! "$BUMP_TYPE" =~ ^[0-9]+\.[0-9]+\.[0-9]+.*$ ]]; then
        echo "Error: Invalid version format '$BUMP_TYPE'. Must be 'major', 'minor', 'patch', or x.y.z"
        exit 1
    fi
    NEW_VERSION="$BUMP_TYPE"
fi

echo "Bumping version from $CURRENT_VERSION to $NEW_VERSION..."

# Update version in Cargo.toml
awk -v old="version = \"$CURRENT_VERSION\"" -v new="version = \"$NEW_VERSION\"" '
    !done && $0 == old {
        print new
        done = 1
        next
    }
    {print}
' "$CARGO_TOML" > "${CARGO_TOML}.tmp" && mv "${CARGO_TOML}.tmp" "$CARGO_TOML"

# Update hardcoded version in CLI (cli_command_handler.rs)
CLI_FILE="$PROJECT_ROOT/src-rust/surfaces/cli_command_handler.rs"
if [ -f "$CLI_FILE" ]; then
    sed -i "s/version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" "$CLI_FILE"
    echo "  Updated CLI version -> $NEW_VERSION"
fi

# Update hardcoded version in TUI (tui_view_controller.rs)
TUI_FILE="$PROJECT_ROOT/src-rust/surfaces/tui_view_controller.rs"
if [ -f "$TUI_FILE" ]; then
    sed -i "s/v$CURRENT_VERSION/v$NEW_VERSION/g" "$TUI_FILE"
    echo "  Updated TUI version -> $NEW_VERSION"
fi

# Update Cargo.lock
echo "Updating Cargo.lock..."
cd "$PROJECT_ROOT"
cargo check > /dev/null 2>&1 || true

# Commit
echo "Staging files..."
git add .

if [ -z "$CUSTOM_MSG" ]; then
    COMMIT_MSG="chore: bump version to v$NEW_VERSION"
else
    COMMIT_MSG="$CUSTOM_MSG"
fi

echo "Committing with message: '$COMMIT_MSG'..."
git commit -m "$COMMIT_MSG"

echo ""
echo "✅ Version bumped to $NEW_VERSION and committed locally."
echo "   (No push, no tag, no release - use release.sh for full release)"
