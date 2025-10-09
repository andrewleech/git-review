Output docs/screenshots/test.gif

Set Width 1400
Set Height 800
Set FontSize 14
Set Theme "Catppuccin Mocha"

# Build release binary (hidden from screenshots)
Hide
Type "cd ~/git-review && cargo build --release 2>&1"
Enter
Sleep 20s
Show

Type "cd ~/git-review"
Enter

Type "clear"
Enter

# Run git-review with --range to show last 6 commits
Type "./target/release/git-review --range HEAD~6..HEAD"
Enter
Sleep 2s

# Screenshot side-by-side mode
Screenshot docs/screenshots/side-by-side.png
Sleep 1s

# Switch to inline mode
Type "i"
Sleep 1s
Screenshot docs/screenshots/inline.png

# Show help
Type "?"
Sleep 1s
Screenshot docs/screenshots/help.png

# Close help and quit
Type "?"
Sleep 300ms
Type "q"
Sleep 500ms
