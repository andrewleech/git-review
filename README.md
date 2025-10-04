# git-review

Terminal-based code review tool for git branches with GitHub-inspired diff UI.

## Features

- **GitHub-inspired diff interface** - Familiar color scheme and layout
- **Responsive layout** - Works on screens from 80x24 to 200x50+
- **Side-by-side and inline diff modes** - Switch between viewing styles
- **Commit log sidebar** - Navigate through branch commits easily
- **Per-hunk context expansion** - Dynamically expand context like on GitHub (planned)
- **Syntax highlighting** - Language-aware code coloring (planned)
- **Review comments** - Add comments to specific lines during review (planned)

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
- `?` - Show help (planned)

### Mouse Support

- Scroll wheel to navigate diff
- Click to select commits (planned)

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

This is an early-stage project. Current implementation includes:

- [x] Git integration (commit log, diff generation)
- [x] Responsive TUI layout
- [x] Commit log pane
- [x] Basic diff view (inline and side-by-side)
- [x] Keyboard navigation
- [ ] Per-hunk context expansion
- [ ] Syntax highlighting
- [ ] Review comments feature
- [ ] Comprehensive tests
- [ ] Documentation and screenshots

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions welcome! Please open an issue or PR on GitHub.

## Authors

- Andrew Leech <andrew@alelec.net>
