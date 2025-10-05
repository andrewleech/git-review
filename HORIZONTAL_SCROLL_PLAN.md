# Horizontal Scrolling Implementation Plan

## Problem
Side-by-side mode truncates long lines with "..." to maintain alignment. This makes code unreadable for review purposes. Users need to see all content.

## Solution
Add synchronized horizontal scrolling where both left and right sides scroll together, similar to vertical scrolling behavior.

## Requirements
1. Both sides must scroll in sync (same horizontal offset)
2. Keyboard controls: h/l (vim) or Left/Right arrow keys
3. Visual indicators when content is hidden left/right
4. Only applies to side-by-side mode (inline can wrap)
5. Reset horizontal scroll when switching commits/files
6. Handle unicode correctly (char boundaries, not bytes)

## Implementation Design

### 1. State Management (app.rs)
Add to App struct:
```rust
pub horizontal_scroll: usize,  // Character offset for side-by-side view
```

Add methods:
```rust
pub fn scroll_horizontal(&mut self, amount: isize) {
    // Scroll left (negative) or right (positive)
    // Bounds check against 0 and max content width
}

pub fn reset_horizontal_scroll(&mut self) {
    self.horizontal_scroll = 0;
}

fn calculate_max_horizontal_scroll(&self) -> usize {
    // Find longest line in current visible content
    // Return max useful scroll offset
}
```

Call reset when:
- Switching commits (select_commit, next_commit, prev_commit)
- Switching files (next_file, prev_file)
- Switching to inline mode

### 2. Input Handling (input.rs)
Add key bindings:
```rust
// Only in side-by-side mode
(KeyCode::Char('h'), KeyModifiers::NONE) | (KeyCode::Left, KeyModifiers::NONE) => {
    if app.config.display.diff_mode == DiffMode::SideBySide {
        app.scroll_horizontal(-4);  // Scroll left by 4 chars
    }
}
(KeyCode::Char('l'), KeyModifiers::NONE) | (KeyCode::Right, KeyModifiers::NONE) => {
    if app.config.display.diff_mode == DiffMode::SideBySide {
        app.scroll_horizontal(4);  // Scroll right by 4 chars
    }
}
```

Scroll amount: 4 characters (configurable in config.toml as `horizontal_scroll_amount`)

### 3. Rendering (side_by_side.rs)
Update function signature:
```rust
pub fn create_side_by_side_lines<'a>(
    app: &App,
    theme: &Theme,
    skip: usize,
    limit: usize,
    max_width: usize,
    horizontal_offset: usize,  // NEW
) -> (Vec<Line<'a>>, Vec<Line<'a>>)
```

Update format_side_line:
```rust
fn format_side_line<'a>(
    hunk_line: &HunkLine, 
    theme: &Theme, 
    is_left: bool, 
    max_width: usize,
    horizontal_offset: usize,  // NEW
) -> Line<'a> {
    // Build full line
    let full_line = format!("{}{}{}", line_num, prefix, hunk_line.content);
    
    // Apply horizontal scroll
    let chars: Vec<char> = full_line.chars().collect();
    let start_idx = horizontal_offset.min(chars.len());
    let end_idx = (start_idx + max_width).min(chars.len());
    
    // Extract visible portion
    let visible: String = chars[start_idx..end_idx].iter().collect();
    
    // Add indicators
    let has_left = horizontal_offset > 0;
    let has_right = end_idx < chars.len();
    
    let display = format!(
        "{}{}{}",
        if has_left { "<" } else { "" },
        visible,
        if has_right { ">" } else { "" }
    );
    
    Line::from(vec![Span::styled(display, style)])
}
```

### 4. UI Updates

#### Footer (footer.rs)
Update shortcuts when in side-by-side mode:
```rust
if app.config.display.diff_mode == DiffMode::SideBySide {
    "... | h/l:scroll-horiz | ..."
} else {
    // Normal footer
}
```

#### Help Dialog (help_dialog.rs)
Add section:
```
Side-by-Side Navigation
  h/l or ←/→  - Scroll horizontally
```

### 5. Configuration (config.rs)
Add to DisplayConfig:
```rust
pub horizontal_scroll_amount: u32,  // Default: 4
```

Update default:
```toml
[display]
horizontal_scroll_amount = 4
```

## Edge Cases

1. **Empty lines**: Skip horizontal scrolling for blank lines
2. **Line numbers**: Keep line numbers visible even when scrolled (don't scroll them)
3. **Very long lines**: Cap max scroll to avoid scrolling into empty space
4. **Unicode**: Use char iteration, not byte slicing
5. **Indicators take space**: Adjust max_width to account for < and > chars
6. **Different line lengths**: Each line independently shows > indicator

## Visual Example

Terminal width: 40 chars per side
Line content: "    const int very_long_variable_name_that_exceeds_width = 42;"

No scroll (offset 0):
```
|     1  const int very_long_variable_>|
```

Scrolled right 10 chars (offset 10):
```
|<    1 int very_long_variable_name_t>|
```

Scrolled right 30 chars (offset 30):
```
|<    1 name_that_exceeds_width = 42; |
```

## Implementation Steps

1. Add horizontal_scroll field and methods to App (~30 lines)
2. Add input handlers for h/l and arrow keys (~15 lines)
3. Update side_by_side.rs with offset parameter and logic (~40 lines)
4. Add reset calls to commit/file navigation (~10 lines)
5. Update footer to show h/l shortcuts conditionally (~10 lines)
6. Update help dialog with horizontal scrolling docs (~5 lines)
7. Add config option for scroll amount (~5 lines)
8. Write tests for horizontal scrolling logic (~30 lines)

Total: ~145 lines across 7 files

## Testing

Manual tests:
- Long lines scroll properly
- Both sides stay in sync
- < and > indicators appear correctly
- Reset works when switching commits
- Unicode handling is correct
- Bounds checking prevents over-scrolling

Unit tests:
- Test horizontal offset calculation
- Test indicator logic
- Test unicode boundary handling

## Alternative Considered: Mouse horizontal scroll

Could add Shift+Scroll for horizontal scrolling, but keyboard-first is simpler and more precise.

## Notes

- Keep inline mode unchanged (it can wrap naturally)
- This feature is side-by-side mode specific
- Must maintain perfect left/right synchronization
- Line numbers column should NOT scroll (stay fixed)
