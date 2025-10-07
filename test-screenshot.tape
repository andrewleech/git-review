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

# Create demo branch from origin/main, then cherry-pick last 6 commits
Type "git checkout -b temp/screenshots origin/main 2>/dev/null || (git branch -D temp/screenshots && git checkout -b temp/screenshots origin/main)"
Enter
Sleep 500ms

Type "git cherry-pick HEAD@{1}~5..HEAD@{1}"
Enter
Sleep 1s

Type "clear"
Enter

# Run git-review (shows cherry-picked commits)
Type "./target/release/git-review"
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

# Cleanup branch
Type "git checkout main && git branch -D temp/screenshots"
Enter
Sleep 300ms
