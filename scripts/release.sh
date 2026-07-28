#!/usr/bin/env bash
# Companion release script
#
# Ensures version files are synced, verifies the version matches the intended
# release, and creates a properly-tagged release.
#
# Usage:
#   ./scripts/release.sh patch        # bump 1.2.8 -> 1.2.9 (new tag companion/v1.2.9)
#   ./scripts/release.sh minor        # bump 1.2.8 -> 1.3.0
#   ./scripts/release.sh major        # bump 1.2.8 -> 2.0.0
#   ./scripts/release.sh --sync-only  # only sync version files (no tag, no bump)
#
# Requires:
#   - clean working tree (or use --allow-dirty)
#   - git configured with push access to origin

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$ROOT_DIR"

# --- Args parsing ---
RELEASE_TYPE=""
SYNC_ONLY=false
ALLOW_DIRTY=false
for arg in "$@"; do
    case "$arg" in
        patch|minor|major)
            RELEASE_TYPE="$arg"
            ;;
        --sync-only)
            SYNC_ONLY=true
            ;;
        --allow-dirty)
            ALLOW_DIRTY=true
            ;;
        -h|--help)
            head -n 20 "$0" | tail -n +3
            exit 0
            ;;
        *)
            echo "Unknown argument: $arg" >&2
            exit 1
            ;;
    esac
done

# --- Pre-flight checks ---
if [[ -z "$RELEASE_TYPE" && "$SYNC_ONLY" == "false" ]]; then
    echo "Error: release type required (patch|minor|major) or --sync-only" >&2
    exit 1
fi

# Check working tree cleanliness (unless --allow-dirty)
if [[ "$ALLOW_DIRTY" == "false" ]]; then
    if ! git diff --quiet HEAD 2>/dev/null; then
        echo "Error: working tree has uncommitted changes. Commit or stash first, or use --allow-dirty." >&2
        exit 1
    fi
fi

# Read current version
CURRENT_VERSION=$(node -p "require('./package.json').version")
echo "Current version: $CURRENT_VERSION"

# --- Sync existing version files ---
echo "--- Syncing version files ---"
npm run sync:version

# --- Bump version if requested ---
if [[ -n "$RELEASE_TYPE" ]]; then
    echo "--- Bumping version ($RELEASE_TYPE) ---"
    # Use standard-version with our release type
    npx standard-version --release-as "$RELEASE_TYPE"

    # Re-read version after bump
    NEW_VERSION=$(node -p "require('./package.json').version")
    echo "New version: $NEW_VERSION"
else
    NEW_VERSION="$CURRENT_VERSION"
    echo "--- No version bump requested ---"
fi

# --- Verify all version files match ---
echo "--- Verifying version sync ---"
EXPECTED_VERSION="$NEW_VERSION"
PKG_VERSION=$(node -p "require('./package.json').version")
CARGO_VERSION=$(grep '^version' src-tauri/Cargo.toml | head -1 | sed -E 's/version = "(.+?)".*/\1/')
TAURI_VERSION=$(node -p "require('./src-tauri/tauri.conf.json').version")

if [[ "$PKG_VERSION" != "$EXPECTED_VERSION" ]]; then
    echo "FAIL: package.json version is $PKG_VERSION, expected $EXPECTED_VERSION" >&2
    exit 1
fi
if [[ "$CARGO_VERSION" != "$EXPECTED_VERSION" ]]; then
    echo "FAIL: Cargo.toml version is $CARGO_VERSION, expected $EXPECTED_VERSION" >&2
    exit 1
fi
if [[ "$TAURI_VERSION" != "$EXPECTED_VERSION" ]]; then
    echo "FAIL: tauri.conf.json version is $TAURI_VERSION, expected $EXPECTED_VERSION" >&2
    exit 1
fi

echo "OK: All version files agree on $EXPECTED_VERSION"

if [[ "$SYNC_ONLY" == "true" ]]; then
    echo "--- Done (sync-only) ---"
    exit 0
fi

# --- Push commits ---
TAG="companion/v$NEW_VERSION"

# Check if tag already exists
if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "FAIL: tag $TAG already exists" >&2
    exit 1
fi

echo "--- Pushing commits ---"
git push origin main

echo "--- Creating tag $TAG ---"
git tag "$TAG"
git push origin "$TAG"

echo ""
echo "✅ Release $NEW_VERSION complete!"
echo "   Tag: $TAG"
echo "   Build will trigger: https://github.com/ManotLuijiu/amos-companion/actions"
echo ""
echo "Verify after build completes:"
echo "  - package.json shows $EXPECTED_VERSION"
echo "  - Header in app shows v$EXPECTED_VERSION"
echo "  - R2 bucket has AMOS_Companion_${NEW_VERSION}_*"
