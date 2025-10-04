#!/bin/bash
set -euo pipefail

# Rust release script - atomically bumps version and creates tag
# Usage: ./release.sh [patch|minor|major|VERSION]

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_usage() {
    echo "Usage: $0 [patch|minor|major|VERSION]"
    echo "Examples:"
    echo "  $0 patch    # 0.2.0 -> 0.2.1"
    echo "  $0 minor    # 0.2.0 -> 0.3.0"
    echo "  $0 major    # 0.2.0 -> 1.0.0"
    echo "  $0 1.2.3    # Set to specific version"
}

bump_version() {
    local current_version="$1"
    local bump_type="$2"

    IFS='.' read -ra VERSION_PARTS <<< "$current_version"
    local major="${VERSION_PARTS[0]}"
    local minor="${VERSION_PARTS[1]}"
    local patch="${VERSION_PARTS[2]}"

    case "$bump_type" in
        patch)
            echo "$major.$minor.$((patch + 1))"
            ;;
        minor)
            echo "$major.$((minor + 1)).0"
            ;;
        major)
            echo "$((major + 1)).0.0"
            ;;
        *)
            # Assume it's a specific version
            echo "$bump_type"
            ;;
    esac
}

validate_version() {
    local version="$1"
    if ! [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo -e "${RED}Error: Invalid version format '$version'. Expected: X.Y.Z${NC}"
        exit 1
    fi
}

main() {
    # Check if we're in a git repository
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        echo -e "${RED}Error: Not in a git repository${NC}"
        exit 1
    fi

    # Check for uncommitted changes
    if ! git diff-index --quiet HEAD --; then
        echo -e "${RED}Error: You have uncommitted changes. Please commit or stash them first.${NC}"
        exit 1
    fi

    # Check if we're on main branch
    current_branch=$(git rev-parse --abbrev-ref HEAD)
    if [[ "$current_branch" != "main" ]]; then
        echo -e "${YELLOW}Warning: You're not on the main branch (current: $current_branch)${NC}"
        read -p "Continue anyway? [y/N]: " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "Aborted."
            exit 1
        fi
    fi

    # Parse arguments
    if [[ $# -eq 0 ]]; then
        print_usage
        exit 1
    fi

    local bump_type="$1"

    # Get current version from Cargo.toml
    local current_version
    current_version=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

    if [[ -z "$current_version" ]]; then
        echo -e "${RED}Error: Could not extract version from Cargo.toml${NC}"
        exit 1
    fi

    echo -e "${GREEN}Current version: $current_version${NC}"

    # Calculate new version
    local new_version
    new_version=$(bump_version "$current_version" "$bump_type")
    validate_version "$new_version"

    echo -e "${GREEN}New version: $new_version${NC}"

    # Confirm with user
    read -p "Continue with release? [y/N]: " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 1
    fi

    # Update version in Cargo.toml
    echo -e "${YELLOW}Updating Cargo.toml...${NC}"
    sed -i "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml

    # Verify the change worked
    local updated_version
    updated_version=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    if [[ "$updated_version" != "$new_version" ]]; then
        echo -e "${RED}Error: Failed to update version in Cargo.toml${NC}"
        exit 1
    fi

    # Update Cargo.lock
    echo -e "${YELLOW}Updating Cargo.lock...${NC}"
    cargo check --quiet

    # Create commit
    echo -e "${YELLOW}Creating commit...${NC}"
    git add Cargo.toml Cargo.lock
    git commit -m "bump version to $new_version"

    # Create tag
    local tag_name="v$new_version"
    echo -e "${YELLOW}Creating tag $tag_name...${NC}"
    git tag -a "$tag_name" -m "Release $new_version"

    # Show what was done
    echo -e "${GREEN}✓ Version bumped: $current_version -> $new_version${NC}"
    echo -e "${GREEN}✓ Commit created${NC}"
    echo -e "${GREEN}✓ Tag created: $tag_name${NC}"
    echo
    echo -e "${YELLOW}To complete the release:${NC}"
    echo "  git push origin main"
    echo "  git push origin $tag_name"
    echo
    echo -e "${YELLOW}Or push both at once:${NC}"
    echo "  git push origin main --tags"
}

main "$@"