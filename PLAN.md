# git-review Implementation Plan

## Project Overview

**git-review** is a terminal-based code review tool for inspecting git branch changes before pushing to a pull request. It provides a GitHub-inspired diff interface optimized for terminal environments, from small screens (80x24) to large displays (200x50+).

**Repository**: https://github.com/andrewleech/git-review

## Core Requirements

### Technical Stack
- **Rust Version**: 1.90.0 (MSRV: 1.74.0)
- **TUI Framework**: ratatui 0.29.0 + crossterm 0.29.0
- **Git Integration**: git2 0.20.2
- **Syntax Highlighting**: syntect 5.2.0
- **Configuration**: serde 1.0.228 + toml 0.9.7
- **Error Handling**: anyhow 1.0.99
- **CLI**: clap 4.4

### Development Constraints
- **Maximum file size**: 350 lines per source file
- **Refactoring policy**: Split files when approaching line limit
- **Focus**: Keep modules single-purpose and maintainable

## Architecture

### Project Structure
```
git-review/
├── PLAN.md                     # This file
├── CLAUDE.md                   # Project guidance for Claude Code
├── README.md                   # User documentation
├── LICENSE                     # MIT license
├── release.sh                  # Version bump script
├── Cargo.toml                  # Package manifest
├── rustfmt.toml                # Code formatting config
├── .gitignore
├── .github/
│   └── workflows/
│       └── ci-cd.yml           # Full CI/CD pipeline
├── src/
│   ├── main.rs                 # Entry point
│   ├── app.rs                  # Application state
│   ├── config.rs               # Config file management
│   ├── input.rs                # Event handling
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── layout.rs           # Responsive sizing
│   │   ├── log_pane.rs         # Commit list
│   │   ├── diff_view.rs        # Diff rendering
│   │   ├── hunk_expander.rs    # Context expansion UI
│   │   ├── comment_dialog.rs   # Comment input dialog
│   │   ├── header.rs           # Top bar
│   │   ├── footer.rs           # Shortcuts bar
│   │   └── theme.rs            # Colors
│   ├── comments.rs             # Comment storage and file I/O
│   └── git/
│       ├── mod.rs
│       ├── commits.rs          # Log extraction
│       ├── diff.rs             # Diff generation
│       ├── diff_parser.rs      # Hunk parsing
│       └── branch.rs           # Base detection
├── tests/
│   └── integration_tests.rs
└── docs/                       # GitHub Pages
    ├── index.html
    ├── usage.html
    ├── screenshots/
    └── assets/
```

## Feature Specifications

### 1. GitHub-Inspired Diff UI

**Core Elements**:
- Split-pane interface: log sidebar + diff view
- Responsive layout adapting to terminal size
- Side-by-side and inline diff modes
- Syntax highlighting with language detection
- Line numbers (old/new) for all changes

**Color Scheme** (GitHub-inspired):
- Additions: Green background/text
- Deletions: Red background/text
- Context: Gray text
- Syntax colors: Based on syntect theme

### 2. Per-Hunk Context Expansion

**Behavior** (GitHub-style):
- Show limited context by default (configurable, default: 8 lines)
- Display expand buttons above/below each hunk when more context available
- Format: `"↑ Expand 8 more lines ↑"` (adaptive to available lines)
- Allow progressive expansion until full file shown
- Track expansion state per-hunk during session

**Implementation**:
```rust
struct HunkExpansion {
    file_path: String,
    hunk_index: usize,
    lines_above: usize,    // Extra lines expanded above default
    lines_below: usize,    // Extra lines expanded below default
}
```

**User Interaction**:
- Click expand button (mouse)
- Press `e` to expand below cursor
- Press `E` to expand above cursor
- Expansion increments from config (default: 8 lines)

### 3. Responsive Layout

**Terminal Size Detection**:
- Use crossterm to query dimensions on startup and resize events
- Recalculate layout dynamically

**Layout Calculations**:

**Small (80 cols)**:
- Log pane: 20 cols (25%)
- Diff area: 58 cols (75%)
- Side-by-side: 28|2|28 (old|gutter|new)

**Medium (120 cols)**:
- Log pane: 30 cols (25%)
- Diff area: 88 cols (75%)
- Side-by-side: 43|2|43

**Large (200 cols)**:
- Log pane: 40 cols (20%)
- Diff area: 158 cols (80%)
- Side-by-side: 78|2|78

**Height (24 lines minimum)**:
- Header: 1 line
- Content: height - 2
- Footer: 1 line

### 4. Log Pane

**Display**:
- Commit hash (short: 7 chars)
- Commit message (first line, truncated to fit)
- Author name (truncated)
- Relative date (e.g., "2 hours ago")
- Selection highlight

**Interaction**:
- Navigate with `j`/`k` or arrow keys
- Select with Enter or click
- Toggle visibility with Space

**Data Source**:
- Git log from current branch to main/master
- Auto-detect base branch (try origin/main, origin/master, main, master)

### 5. Diff View

**Side-by-Side Mode** (default):
- Two columns: old file | new file
- Align changed lines horizontally
- Show line numbers for both versions
- Gutter between columns for visual separation

**Inline Mode**:
- Single column view
- Prefix lines: ` ` (context), `-` (removed), `+` (added)
- Line numbers show old/new appropriately

**File Navigation**:
- Show file path in header
- Navigate between files with `[`/`]` keys
- Display file stats: `+X -Y lines`

### 6. Input Handling

**Keyboard Shortcuts**:
- `q`: Quit application
- `Space`: Toggle log pane visibility
- `s`: Switch to side-by-side mode
- `i`: Switch to inline mode
- `j`/`k` or `↓`/`↑`: Scroll diff
- `n`/`p`: Next/previous commit
- `[`/`]`: Next/previous file
- `e`: Expand context below cursor
- `E`: Expand context above cursor
- `c`: Add/edit comment on current line
- `v`: View comments on current line
- `d`: Delete comment (when viewing comments)
- `?`: Show help

**Mouse Support**:
- Scroll wheel: Navigate diff
- Click: Select commits, click expand buttons
- Drag: Not initially required

### 7. Configuration

**File Location**: `~/.config/git-review/config.toml`

**Schema**:
```toml
[display]
diff_mode = "side-by-side"      # or "inline"
context_lines = 8               # Initial context per hunk
context_expand_increment = 8    # Lines per expansion click
syntax_theme = "github-dark"    # syntect theme name

[ui]
log_pane_width_ratio = 0.25    # % of terminal width (0.0-0.5)
show_line_numbers = true
```

**Behavior**:
- Create with defaults if missing
- Save mode preference on change
- Validate values on load

### 8. Code Review Comments

**Feature**: Click on any line of code to add review comments saved to timestamped text files.

**Behavior**:
- Click on a line in the diff view to add a comment
- Keyboard shortcut: `c` to comment on current line
- Pop-up text input dialog appears
- Enter multi-line comment (Ctrl+D or Ctrl+S to save, Esc to cancel)
- Comments saved to `.git-review/` directory in working tree
- Filename format: `review-YYYY-MM-DD-HHMMSS.txt`

**Comment File Format**:
```
Review Date: 2025-10-04 22:30:15
Commit: abc123f
Branch: feature/new-feature

---

File: src/main.rs
Line: 42 (new)
Comment:
This function should handle the error case more gracefully.
Consider returning a Result instead of unwrap().

---

File: src/app.rs
Line: 105 (old)
Comment:
This logic is unclear. Add documentation explaining why we need this check.
```

**Implementation**:
```rust
struct ReviewComment {
    file_path: String,
    line_number: usize,
    line_type: LineType,  // Old, New, or Context
    comment: String,
    timestamp: DateTime<Local>,
}
```

**UI Elements**:
- Comment indicator: Small marker next to lines with comments
- View comments: Hover or press `v` to view existing comments on a line
- Edit comments: Click again or press `c` to edit existing comment
- Delete comments: Press `d` in comment view to delete

**Storage**:
- Save to `.git-review/review-TIMESTAMP.txt` after each comment
- Append to same file during a review session
- Load and display existing comments from current session
- `.git-review/` added to `.gitignore` automatically

## Git Integration

### Branch Detection

Auto-detect base branch in order:
1. `origin/main`
2. `origin/master`
3. `main`
4. `master`

Exit with error if none found.

### Commit Log Extraction

```rust
// Pseudocode
fn get_commits(repo: &Repository, base: &str) -> Result<Vec<Commit>> {
    let head = repo.head()?;
    let base_oid = repo.revparse_single(base)?.id();

    // Walk from HEAD to base
    let mut revwalk = repo.revwalk()?;
    revwalk.push(head.target())?;
    revwalk.hide(base_oid)?;

    // Collect commit metadata
    // ...
}
```

### Diff Generation with Context

Use git2 to generate diffs with configurable context:
- Initial context from config
- Re-generate with expanded context on demand
- Parse hunk headers to determine available lines for expansion

## UI Implementation Details

### Rendering Pipeline

**Frame Update**:
1. Clear terminal
2. Render header (commit info, file path)
3. Render main area:
   - If log visible: split layout (log pane + diff)
   - If log hidden: full-width diff
4. Render footer (keyboard shortcuts)

**Diff Rendering**:
1. Parse git diff into hunks
2. Apply syntax highlighting per line
3. Check expansion state for each hunk
4. Render expand buttons if context available
5. Render diff lines with appropriate styling

### State Management

```rust
struct AppState {
    commits: Vec<CommitInfo>,
    selected_commit: usize,
    selected_file: usize,
    diff_mode: DiffMode,
    log_pane_visible: bool,
    hunk_expansions: HashMap<HunkId, ExpansionState>,
    scroll_offset: usize,
}
```

## Testing Strategy

### Unit Tests

Test each module independently:
- Git operations: Mock Repository
- Config: Test serialization/deserialization
- Layout: Test size calculations
- Diff parsing: Test various diff formats

### Integration Tests

Use `tempfile` to create test git repositories:
```rust
#[test]
fn test_commit_log_extraction() {
    let temp_repo = setup_test_repo();
    make_commits(&temp_repo, 3);
    let commits = extract_log(&temp_repo);
    assert_eq!(commits.len(), 3);
}
```

### CI Pipeline Tests

GitHub Actions runs:
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- Cross-platform builds (Linux, macOS, Windows)
- Matrix: stable, beta, MSRV (1.74.0)

## Release Process

### Script: release.sh

Copied from git-ignore project:
- Validates git status (clean working directory)
- Warns if not on main branch
- Bumps version in Cargo.toml
- Updates Cargo.lock
- Creates commit: "bump version to X.Y.Z"
- Creates annotated tag: "vX.Y.Z"
- Shows push instructions

**Usage**:
```bash
./release.sh patch  # 0.1.0 -> 0.1.1
./release.sh minor  # 0.1.0 -> 0.2.0
./release.sh major  # 0.1.0 -> 1.0.0
```

### CI/CD Pipeline

On tag push (`v*`):
1. **Test** job: Run full test suite
2. **Security** job: Run `cargo audit`
3. **Build** job: Create binaries for Linux/macOS/Windows
4. **Release** job: Create GitHub release with binaries
5. **Publish** job: Publish to crates.io (requires CRATES_IO_TOKEN)
6. **Test-install** job: Verify `cargo install git-review` works

## Documentation

### GitHub Pages (docs/)

**index.html**:
- Project overview
- Installation instructions
- Quick start guide
- Feature highlights
- Screenshots

**usage.html**:
- Keyboard shortcuts reference
- Configuration options
- Command-line arguments
- Tips and tricks

**screenshots/**:
- Captured using termshot
- Show key features:
  - Log pane view
  - Side-by-side diff
  - Inline diff
  - Context expansion in action
  - Syntax highlighting

### README.md

Contents:
- Project description
- Installation (cargo install, binary downloads)
- Usage examples
- Configuration
- Contributing
- License

## Implementation Phases

### Phase 1: Foundation ✓
- [x] Project structure
- [x] Cargo.toml with dependencies
- [x] Basic main.rs with TUI initialization
- [x] Config file loading/saving
- [x] release.sh and ci-cd.yml setup

### Phase 2: Git Operations ✓
- [x] Repository detection
- [x] Base branch detection
- [x] Commit log extraction
- [x] Diff generation with configurable context

### Phase 3: Basic UI ✓
- [x] Responsive layout calculation
- [x] Log pane rendering
- [x] Basic diff view (inline mode)
- [x] Header and footer

### Phase 4: Advanced Diff ✓
- [x] Side-by-side diff mode
- [x] Syntax highlighting module (not yet integrated)
- [x] Line number display
- [x] File navigation ([ ] keys, scrollable view)

### Phase 5: Context Expansion ✓
- [x] Parse hunk metadata
- [x] Track expansion state (global context level)
- [x] Render expand buttons
- [x] Handle expansion interactions ('e' key)

### Phase 6: Polish ✓
- [x] Mouse support (scroll, click commits)
- [x] Input handling refinement (fixed key modifiers)
- [x] Error handling and messages
- [x] Performance optimization (windowed rendering)

### Phase 7: Documentation & Release
- [x] Write comprehensive tests (22 passing)
- [ ] Create termshot screenshots
- [ ] Build GitHub Pages site
- [ ] Initial release (v0.1.0)

## Maintenance Notes

### Code Quality
- Enforce 350-line limit per file
- Run `cargo fmt` before commits
- Keep clippy warnings at zero
- Document public APIs
- Write tests for new features

### Version Management
- Use semantic versioning
- Tag releases: vX.Y.Z
- Update CHANGELOG.md
- Keep Cargo.toml version synchronized

### CLAUDE.md Updates
- Document development workflow as it evolves
- Note any architectural decisions
- Add troubleshooting guidance
- Keep command examples current

### Code Review Requirement

**MANDATORY**: Before each git commit, perform an independent code review using the principal-code-reviewer agent.

**Process**:
1. Complete implementation of feature or fix
2. Run all tests (`cargo test`)
3. Run quality checks (`cargo fmt --check && cargo clippy`)
4. Launch principal-code-reviewer agent for review
5. Address any issues identified in review
6. Create commit only after review approval

**Review Focus Areas**:
- Code simplicity and maintainability
- Performance implications
- Error handling completeness
- Test coverage adequacy
- Documentation clarity
- Adherence to 350-line file limit
- Potential edge cases or bugs

**Agent Invocation**:
```
Use Task tool with subagent_type: "principal-code-reviewer"
Provide context about the changes and request review
```

This ensures code quality and catches issues early in the development cycle.
