#!/bin/bash
set -euo pipefail

# Automated screenshot capture for git-review documentation
# Requires: ansee (cargo install ansee), script command

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
SCREENSHOTS_DIR="$PROJECT_ROOT/docs/screenshots"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}Git-review automated screenshot capture${NC}"

# Check if ansee is installed
if ! command -v ansee &> /dev/null; then
    echo -e "${YELLOW}Error: ansee not found. Install with: cargo install ansee${NC}"
    exit 1
fi

# Create screenshots directory
mkdir -p "$SCREENSHOTS_DIR"

# Build git-review
echo -e "${YELLOW}Building git-review...${NC}"
cd "$PROJECT_ROOT"
cargo build --release

# Create a temporary demo repository
DEMO_REPO=$(mktemp -d)
echo -e "${YELLOW}Creating demo repository at $DEMO_REPO${NC}"

cd "$DEMO_REPO"
git init
git config user.name "Demo User"
git config user.email "demo@example.com"

# Create initial commit
cat > README.md << 'EOF'
# Sample Project

This is a sample project for demonstrating git-review.

## Features

- Feature A
- Feature B
- Feature C
EOF

git add README.md
git commit -m "Initial commit"

# Create base branch
git branch -M main

# Create feature branch with changes
git checkout -b feature/demo

# Make some changes for side-by-side diff
cat > README.md << 'EOF'
# Sample Project

This is a demo project for showcasing git-review's capabilities.

## Features

- Enhanced Feature A with new functionality
- Feature B (improved performance)
- Feature C
- NEW: Feature D with advanced options

## Usage

Run the application with:
```bash
./app --help
```
EOF

# Add a new file
cat > src/main.rs << 'EOF'
fn main() {
    println!("Hello, world!");

    // Feature A implementation
    let result = calculate_something(42);
    println!("Result: {}", result);
}

fn calculate_something(input: i32) -> i32 {
    input * 2 + 10
}
EOF

mkdir -p src
git add .
git commit -m "Add feature implementation and improve documentation"

# Make another commit
cat >> src/main.rs << 'EOF'

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate() {
        assert_eq!(calculate_something(42), 94);
    }
}
EOF

git add src/main.rs
git commit -m "Add unit tests"

echo -e "${GREEN}Demo repository created with 2 commits on feature/demo branch${NC}"

# Function to capture screenshot with specific terminal size
capture_screenshot() {
    local name=$1
    local cols=${2:-120}
    local rows=${3:-30}
    local output="$SCREENSHOTS_DIR/${name}.txt"

    echo -e "${YELLOW}Capturing: $name (${cols}x${rows})${NC}"

    # Use script to capture with timeout and specific terminal size
    # This captures the raw ANSI output
    export COLUMNS=$cols
    export LINES=$rows

    # Run git-review with timeout (5 seconds should be enough for capture)
    timeout 5s script -qec "$PROJECT_ROOT/target/release/git-review --path $DEMO_REPO" "$output" || true

    # Clean up script command artifacts
    sed -i 's/\r//g' "$output"

    echo -e "${GREEN}Captured to: $output${NC}"
}

# Note: Actual screenshot capture would require:
# 1. Sending specific keypresses to git-review
# 2. Timing the captures correctly
# 3. Converting ANSI output to PNG with ansee

echo -e "${YELLOW}
NOTE: This script creates a demo repository and prepares the environment.
To capture actual screenshots, you need to:

1. Run git-review interactively in the demo repo
2. Use 'script' command with appropriate timing
3. Extract specific frames from the captured output
4. Convert to PNG with: ansee input.txt -o output.png

Demo repository location: $DEMO_REPO
Keep this directory for manual screenshot capture.
${NC}"

echo -e "${GREEN}Setup complete!${NC}"
echo "Run git-review in demo repo: $PROJECT_ROOT/target/release/git-review --path $DEMO_REPO"
