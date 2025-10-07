# Automated Screenshot Capture for git-review

## Investigation Summary

This document outlines the investigation into automated/headless screenshot capture for terminal applications, specifically for git-review documentation.

## Tools Evaluated

### 1. VHS (by Charm)
**Best for: Automated, scriptable terminal recordings**

- **Pros**:
  - Scriptable with "tape" files
  - Can simulate keypresses and timing
  - Outputs GIF and PNG
  - Designed specifically for TUI demos
  - Works headless with ttyd
- **Cons**:
  - Requires: ttyd, ffmpeg, Go toolchain
  - Not available in Ubuntu repos
  - Heavier dependency stack

**Installation**:
```bash
# Requires Go, ttyd, ffmpeg
go install github.com/charmbracelet/vhs@latest
```

**Usage**:
```bash
# Create a tape file
vhs new demo.tape

# Edit tape file with commands:
Output demo.gif
Set Width 1200
Set Height 600
Type "git-review"
Enter
Sleep 2s
Type "j"
Sleep 500ms

# Generate
vhs demo.tape
```

### 2. asciinema + agg
**Best for: Recording sessions and converting to GIF**

- **Pros**:
  - asciinema widely available (Ubuntu repos)
  - agg written in Rust (cargo install agg)
  - Good quality GIF output
- **Cons**:
  - Requires manual interaction for recording
  - Output is GIF (not static PNG)
  - Less control over exact frames

**Installation**:
```bash
sudo apt install asciinema
cargo install --git https://github.com/asciinema/agg
```

**Usage**:
```bash
# Record session
asciinema rec demo.cast

# Convert to GIF
agg demo.cast demo.gif
```

### 3. ansee + script
**Best for: Static PNG screenshots from ANSI output**

- **Pros**:
  - Written in Rust
  - Outputs static PNG
  - Good font customization
  - Simple conversion
- **Cons**:
  - Requires capturing raw ANSI output first
  - Manual timing/interaction needed
  - Doesn't simulate keypresses

**Installation**:
```bash
cargo install ansee
```

**Usage**:
```bash
# Capture terminal session
script -q output.txt

# Run your app, then exit script (Ctrl+D)

# Convert to PNG
ansee output.txt -o screenshot.png \
  --font-size 14 \
  --line-height 1.4
```

### 4. termtosvg
**Best for: SVG animations**

- **Pros**:
  - Available in Ubuntu repos
  - Creates clean SVG animations
  - Can be converted to PNG
- **Cons**:
  - SVG format (needs rasterization)
  - Interactive recording only

**Installation**:
```bash
sudo apt install termtosvg
```

## Recommended Approach

### For Static PNG Documentation Screenshots:

**Option A: Manual capture with ansee (Simplest)**
1. Use `script` command to record terminal session
2. Manually interact with git-review
3. Convert captured ANSI output to PNG with ansee
4. Edit/crop as needed

```bash
# 1. Start recording
script -q capture.txt

# 2. Run git-review and navigate to desired view
git-review

# 3. Exit (Ctrl+C to quit git-review, Ctrl+D to end script)

# 4. Convert to PNG
ansee capture.txt -o screenshot.png --font-size 13 --line-height 1.3
```

**Option B: VHS automation (Best quality, requires setup)**
1. Install VHS and dependencies (Go, ttyd, ffmpeg)
2. Create tape files for each screenshot scenario
3. Generate screenshots automatically in CI

```tape
# side-by-side-demo.tape
Output side-by-side.gif
Set Width 1400
Set Height 800
Set FontSize 14

Type "git-review"
Enter
Sleep 2s

# Navigate to interesting diff
Type "n"
Sleep 500ms
Type "PgDn"
Sleep 1s

# Ensure side-by-side mode
Type "s"
Sleep 500ms

# Final frame
Screenshot side-by-side.png
Sleep 2s
```

### For GIF Animations:

**Best: asciinema + agg**
- Record actual usage with asciinema
- Convert to high-quality GIF with agg
- Add to GitHub README

```bash
asciinema rec -t "git-review demo" demo.cast
agg demo.cast demo.gif
```

## Current Status

### Installed Tools:
- ✓ script (built-in)
- ⏳ ansee (compiling, ~5 min)
- ✗ VHS (requires Go, ttyd, ffmpeg)
- ✗ asciinema (not installed yet)
- ✗ agg (not installed yet)

### Created Resources:
- `/home/corona/git-review/scripts/capture-screenshots.sh` - Setup script for demo repository
- This documentation

## Next Steps

1. **Wait for ansee installation to complete**
2. **Test manual screenshot workflow**:
   ```bash
   # Create demo repo
   ./scripts/capture-screenshots.sh

   # Capture screenshots manually
   cd demo-repo
   script -q capture.txt
   ../target/release/git-review
   # Navigate, then Ctrl+C and Ctrl+D

   ansee capture.txt -o screenshot.png
   ```

3. **Consider VHS for fully automated approach** (requires dependencies)

4. **Document specific screenshots needed**:
   - Side-by-side diff view
   - Inline diff view
   - Help dialog
   - Log pane visible
   - Horizontal scrolling indicators
   - Context expansion

## Implementation Notes

### Challenges with TUI Screenshot Automation:

1. **Interaction simulation**: TUI apps need actual keypresses, not just stdin
2. **Timing**: Need to capture at exact moment after rendering
3. **Terminal initialization**: Apps initialize alternate screen buffer
4. **Frame extraction**: Hard to extract single frame from continuous session

### Why VHS is Ideal:

- Uses ttyd to run terminal in browser
- Can send keyboard events programmatically
- Takes screenshots at specific points
- Built for this exact use case

### Compromise Approach:

For v0.1.0 release, screenshots can be added post-release:
1. Install minimal tools (ansee only)
2. Manually capture key screenshots
3. Good enough for initial documentation
4. Can upgrade to VHS automation later for v0.2.0

## Cost-Benefit Analysis

| Approach | Setup Time | Automation | Quality | Maintenance |
|----------|-----------|------------|---------|-------------|
| Manual + ansee | 10 min | Low | Good | Medium |
| asciinema + agg | 20 min | Medium | Good | Low |
| VHS full automation | 60 min | High | Excellent | Low |

**Recommendation for now**: Manual + ansee (pragmatic, quick)
**Recommendation for future**: VHS automation in CI (scalable, repeatable)
