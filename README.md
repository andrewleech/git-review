# git-review

Terminal-based code review tool for git branches with GitHub-inspired diff UI.

## Features

### Implemented
- **GitHub-inspired diff interface** - Familiar color scheme and layout
- **Responsive layout** - Works on screens from 80x24 to 200x50+
- **Side-by-side and inline diff modes** - Switch between viewing styles
- **Commit log sidebar** - Navigate through branch commits easily
- **Interactive help dialog** - Press `?` for keyboard shortcuts
- **Per-hunk context expansion UI** - Expand buttons shown (full integration pending)
- **Mouse support** - Scroll wheel navigation

### In Progress
- **Syntax highlighting** - Module ready, UI integration pending
- **Review comments** - Persistence system complete, UI integration pending
- **Context expansion** - Button UI complete, diff regeneration pending

## Installation

### From source

```bash
git clone https://github.com/andrewleech/git-review
cd git-review
cargo build --release
cargo install --path .
```

### From crates.io (after release)

```bash
cargo install git-review
```

## Usage

Navigate to a git repository and run:

```bash
git-review
```

### Options

```
Options:
  -p, --path <PATH>     Path to git repository (defaults to current directory)
  -b, --base <BRANCH>   Base branch to compare against
  -c, --context <LINES> Initial context lines for diffs [default: 8]
  -h, --help            Print help
  -V, --version         Print version
```

### Keyboard Shortcuts

- `q` - Quit application
- `space` - Toggle commit log pane visibility
- `s` - Switch to side-by-side diff mode
- `i` - Switch to inline diff mode
- `j/k` or `↓/↑` - Scroll diff view
- `n/p` - Next/previous commit
- `[/]` - Next/previous file
- `?` - Show help dialog
- `Esc` - Close help dialog

### Mouse Support

- Scroll wheel to navigate diff

## Configuration

Configuration is stored at `~/.config/git-review/config.toml`:

```toml
[display]
diff_mode = "side-by-side"      # or "inline"
context_lines = 8               # Initial context per hunk
context_expand_increment = 8    # Lines added per expansion
syntax_theme = "base16-ocean.dark"

[ui]
log_pane_width_ratio = 0.25    # % of terminal width
show_line_numbers = true
```

## Development

See [CLAUDE.md](CLAUDE.md) and [PLAN.md](PLAN.md) for development documentation.

### Build and Test

```bash
cargo build
cargo test
cargo run
```

### Code Quality

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
```

## Release Process

Use the included `release.sh` script:

```bash
./release.sh patch  # 0.1.0 -> 0.1.1
./release.sh minor  # 0.1.0 -> 0.2.0
./release.sh major  # 0.1.0 -> 1.0.0
```

## Project Status

**Version**: 0.1.0 (Early Development)

### Completed
- [x] Git integration (commit log, diff generation, branch detection)
- [x] Responsive TUI layout (80x24 to 200x50+)
- [x] Commit log pane with selection
- [x] Diff view (inline and side-by-side modes)
- [x] Keyboard and mouse navigation
- [x] Help dialog
- [x] Per-hunk context expansion UI (buttons visible)
- [x] Syntax highlighting module (prepared)
- [x] Review comments system (backend ready)
- [x] Integration and unit tests (22 tests passing)
- [x] CI/CD pipeline (GitHub Actions)
- [x] Release automation (release.sh)

### In Progress
- [ ] Wire up context expansion to regenerate diffs
- [ ] Integrate syntax highlighting into diff view
- [ ] Integrate comment dialog UI
- [ ] Add comprehensive error messages to UI

### Future Enhancements
- [ ] GitHub Pages documentation site
- [ ] Terminal screenshots with termshot
- [ ] Performance optimization for large diffs
- [ ] File tree view
- [ ] Search within diffs

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions welcome! Please open an issue or PR on GitHub.

## Authors

- Andrew Leech <andrew@alelec.net>
