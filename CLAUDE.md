# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**git-review** is a terminal-based code review tool for git branches. It provides a GitHub-inspired diff interface optimized for reviewing changes before pushing to a PR. See **PLAN.md** for comprehensive implementation details.

## Development Constraints

**CRITICAL**: Maximum 350 lines per source file. Refactor into multiple files when approaching this limit.

## Quick Reference

### Build and Test
```bash
# Build
cargo build
cargo build --release

# Run
cargo run

# Test all
cargo test

# Test specific
cargo test --lib                     # Unit tests only
cargo test --test integration_tests  # Integration tests only
cargo test --doc                     # Doc tests

# Quality checks (matches CI)
cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test
```

### Code Quality
```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Linting
cargo clippy --all-targets --all-features -- -D warnings

# Fix clippy suggestions
cargo clippy --fix --all-targets --all-features
```

## Architecture

Refer to PLAN.md for detailed architecture. Key points:

### Module Organization
```
src/
├── main.rs              # Entry point (CLI args, TUI initialization)
├── app.rs               # Application state management
├── config.rs            # ~/.config/git-review/config.toml handling
├── input.rs             # Keyboard/mouse event processing
├── ui/                  # All UI rendering
│   ├── layout.rs        # Responsive terminal sizing
│   ├── log_pane.rs      # Commit list sidebar
│   ├── diff_view.rs     # Main diff display
│   ├── hunk_expander.rs # Context expansion UI
│   ├── header.rs        # Top bar (commit info)
│   ├── footer.rs        # Bottom bar (shortcuts)
│   └── theme.rs         # Color scheme
└── git/                 # Git integration
    ├── commits.rs       # Extract commit log
    ├── diff.rs          # Generate diffs
    ├── diff_parser.rs   # Parse hunk metadata
    └── branch.rs        # Detect base branch
```

### Key Design Patterns

**Responsive Layout**: Calculates UI dimensions dynamically based on terminal size (crossterm). Supports 80x24 to 200x50+.

**Per-Hunk Context Expansion**: Tracks expansion state per hunk during session. Renders expand buttons (GitHub-style) when more context available. Re-fetches git diff with adjusted context on demand.

**State Management**: Single `AppState` struct owns all application state (commits, selection, expansions, scroll position, config).

**Error Handling**: Use `anyhow` throughout for error propagation. Provide user-friendly error messages in TUI.

## Dependencies

Core runtime dependencies (see Cargo.toml for versions):
- `ratatui` + `crossterm`: TUI framework
- `git2`: Git operations via libgit2 bindings
- `syntect`: Syntax highlighting
- `serde` + `toml`: Configuration persistence
- `anyhow`: Error handling
- `clap`: CLI argument parsing

Dev dependencies:
- `assert_cmd`: CLI integration testing
- `predicates`: Test assertions
- `tempfile`: Temporary test repositories

## Release Process

Use `./release.sh` for version management:

```bash
# Semantic version bumps
./release.sh patch  # 0.1.0 -> 0.1.1 (bug fixes)
./release.sh minor  # 0.1.0 -> 0.2.0 (new features)
./release.sh major  # 0.1.0 -> 1.0.0 (breaking changes)

# Or specific version
./release.sh 1.2.3
```

**Script Safety**:
- Validates clean git state
- Warns if not on main branch
- Updates Cargo.toml and Cargo.lock
- Creates commit and annotated tag atomically
- Shows push instructions

**Complete Release**:
```bash
./release.sh patch
git push origin main --tags
```

Pushing tags triggers CI/CD:
- Cross-platform builds (Linux/macOS/Windows)
- GitHub release with binaries
- crates.io publish (requires CRATES_IO_TOKEN secret)
- Validation (tag version matches Cargo.toml)

## Git Integration

### Base Branch Detection

Tries in order: `origin/main`, `origin/master`, `main`, `master`. Exits with error if none found.

### Commit Extraction

Use git2 `revwalk` to get commits between HEAD and base branch.

### Diff Generation

Generate diffs with configurable context lines. Parse hunk headers to identify expandable regions.

## Configuration

User config at `~/.config/git-review/config.toml`:

```toml
[display]
diff_mode = "side-by-side"      # or "inline"
context_lines = 8               # Initial context per hunk
context_expand_increment = 8    # Lines added per expansion
syntax_theme = "github-dark"

[ui]
log_pane_width_ratio = 0.25    # % of terminal width
show_line_numbers = true
```

Create with defaults if missing. Persist mode changes.

## Testing Strategy

### Unit Tests

In-module tests (`#[cfg(test)]`) for:
- Config serialization
- Layout calculations
- Diff parsing
- Git operations (use git2 test helpers)

### Integration Tests

`tests/integration_tests.rs` using `assert_cmd` and `tempfile`:
- Create temporary git repos
- Run CLI with various arguments
- Verify output and behavior

## Implementation Progress

**Current Status**: Version 0.1.0 - Core functionality complete, ready for release

### Completed Features
- [x] Project structure and dependencies
- [x] Release automation (release.sh, ci-cd.yml)
- [x] Git operations (commits, diff, branch detection)
- [x] Configuration management with TOML persistence
- [x] TUI initialization and event loop
- [x] Responsive layout system (80x24 to 200x50+)
- [x] Log pane with commit selection (keyboard + mouse)
- [x] Diff view (side-by-side and inline modes)
- [x] Context expansion (press 'e' to expand, 'r' to reset)
- [x] All files shown in scrollable inline view
- [x] Syntax highlighting module (prepared, not yet integrated)
- [x] Review comments system (backend complete)
- [x] Keyboard/mouse input handling
- [x] Header and footer UI
- [x] Help dialog (press ?)
- [x] Tests (22 passing: 17 unit + 5 integration)
- [x] Documentation (README, PLAN.md, CLAUDE.md)

### Recent Improvements
- Fixed mouse click race condition on terminal resize
- Added scroll bounds checking to prevent empty space
- Optimized side-by-side rendering memory usage (windowed rendering)
- Improved file line count estimation
- Implemented context expansion with 'e' and 'r' keys
- Fixed side-by-side alignment with proper line pairing
- Added horizontal scrolling for long lines (h/l keys with visual indicators)
- Changed file navigation to PgUp/PgDn keys
- Increased log pane width to 35% for better commit message visibility
- Fixed help dialog to block background scrolling
- Moved ? shortcut to start of footer for small screen visibility

### Release Readiness (v0.1.0)
- [x] Core functionality complete (all features working)
- [x] 22 tests passing (17 unit + 5 integration)
- [x] Documentation updated (README.md, PLAN.md, CLAUDE.md)
- [x] GitHub Pages site created (index.html, usage.html)
- [x] CI/CD pipeline configured (.github/workflows/ci-cd.yml)
- [x] Release script ready (release.sh)

### Pending Features (Future Versions)
- [ ] Integrate syntax highlighting into diff rendering (module exists)
- [ ] Add comment dialog UI (backend complete)
- [ ] Connect comment system to keyboard shortcuts ('c', 'v', 'd')
- [ ] Terminal screenshots with termshot or vhs

### Code Quality Metrics
- **Files**: 21 Rust source files
- **Total Lines**: ~2,400 lines of code
- **Largest File**: 315 lines (diff_view.rs) - 10% under 350-line limit
- **Test Coverage**: 22 tests covering core functionality
- **Build Time**: ~4s debug, ~2min release (due to syntect/git2)

## Common Tasks

### Adding a New UI Component

1. Create file in `src/ui/` (keep under 350 lines)
2. Implement rendering function taking `&mut Frame` and necessary state
3. Wire into main render loop in appropriate module
4. Add keyboard shortcuts in `input.rs` if needed

### Adding Git Functionality

1. Add function in appropriate `src/git/` module
2. Return `Result<T, anyhow::Error>` for error handling
3. Write unit tests with git2 test helpers
4. Integration test with temporary repo if needed

### Debugging TUI

Enable mouse capture can interfere with terminal. If app crashes without cleanup:
```bash
# Reset terminal
reset

# Or manually
tput cnorm  # Show cursor
stty sane   # Reset terminal settings
```

## File Size Monitoring

Check file sizes during development:
```bash
find src -name '*.rs' -exec wc -l {} \; | sort -rn
```

Refactor files approaching 350 lines.

## Resources

- **PLAN.md**: Detailed implementation specifications
- **ratatui docs**: https://docs.rs/ratatui
- **git2 docs**: https://docs.rs/git2
- **syntect docs**: https://docs.rs/syntect
