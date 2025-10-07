Output docs/screenshots/test.gif
Output docs/screenshots/test.png

Set Width 1400
Set Height 800
Set FontSize 14
Set Theme "Catppuccin Mocha"

Type "export PATH=$PATH:~/.cargo/bin"
Enter
Sleep 500ms

Type "cd /tmp/tmp.cqQtc7HF2G"
Enter
Sleep 500ms

Type "../../../home/corona/git-review/target/release/git-review"
Enter

# Wait for app to load
Sleep 3s

# Take a screenshot of side-by-side mode
Screenshot docs/screenshots/side-by-side.png
Sleep 1s

# Switch to inline mode
Type "i"
Sleep 1s

# Screenshot inline mode
Screenshot docs/screenshots/inline.png
Sleep 1s

# Show help dialog
Type "?"
Sleep 1s

# Screenshot help
Screenshot docs/screenshots/help.png
Sleep 1s

# Close help
Type "?"
Sleep 500ms

# Quit
Type "q"
Sleep 500ms
