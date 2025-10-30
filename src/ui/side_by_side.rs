use crate::app::App;
use crate::git::{HunkLine, LineType};
use crate::ui::theme::Theme;
use ratatui::text::{Line, Span};

/// Create side-by-side diff lines (left: old/removed, right: new/added)
/// Only creates lines within the visible window to save memory on large diffs
/// Aligns removed and added lines side-by-side for proper comparison
/// Lines are horizontally scrolled by horizontal_offset and truncated to max_width
pub fn create_side_by_side_lines<'a>(
    app: &App,
    theme: &Theme,
    skip: usize,
    limit: usize,
    max_width: usize,
    horizontal_offset: usize,
) -> (Vec<Line<'a>>, Vec<Line<'a>>) {
    let mut left_lines = Vec::with_capacity(limit);
    let mut right_lines = Vec::with_capacity(limit);
    let mut current_line = 0;
    let end_line = skip + limit;

    if let Some(file) = app.selected_file() {
        // Show hunks
        for hunk in &file.hunks {
            // Hunk header on both sides (scrolled and truncated if needed)
            if current_line >= skip && current_line < end_line {
                let mut header =
                    apply_horizontal_scroll(&hunk.header, horizontal_offset, max_width);

                // Ensure header fits exactly within max_width
                let header_len = header.chars().count();
                match header_len.cmp(&max_width) {
                    std::cmp::Ordering::Greater => {
                        header = header.chars().take(max_width).collect();
                    }
                    std::cmp::Ordering::Less => {
                        header.push_str(&" ".repeat(max_width - header_len));
                    }
                    std::cmp::Ordering::Equal => {}
                }

                left_lines.push(Line::from(vec![Span::styled(
                    header.clone(),
                    theme.context_style(),
                )]));
                right_lines.push(Line::from(vec![Span::styled(
                    header,
                    theme.context_style(),
                )]));
            }
            current_line += 1;
            if current_line >= end_line {
                break;
            }

            // Process hunk lines with proper alignment
            let mut i = 0;
            while i < hunk.lines.len() && current_line < end_line {
                let hunk_line = &hunk.lines[i];

                match hunk_line.line_type {
                    LineType::Context => {
                        // Context appears on both sides
                        if current_line >= skip {
                            let left_line = format_side_line(
                                hunk_line,
                                theme,
                                true,
                                max_width,
                                horizontal_offset,
                            );
                            let right_line = format_side_line(
                                hunk_line,
                                theme,
                                false,
                                max_width,
                                horizontal_offset,
                            );
                            left_lines.push(left_line);
                            right_lines.push(right_line);
                        }
                        current_line += 1;
                        i += 1;
                    }
                    LineType::Removed => {
                        // Collect consecutive removed lines
                        let removed_start = i;
                        while i < hunk.lines.len() && hunk.lines[i].line_type == LineType::Removed {
                            i += 1;
                        }

                        // Collect consecutive added lines (if any follow)
                        let added_start = i;
                        while i < hunk.lines.len() && hunk.lines[i].line_type == LineType::Added {
                            i += 1;
                        }

                        let removed_lines = &hunk.lines[removed_start..added_start];
                        let added_lines = &hunk.lines[added_start..i];

                        // Pair up removed and added lines side-by-side
                        let max_len = removed_lines.len().max(added_lines.len());
                        for j in 0..max_len {
                            if current_line >= end_line {
                                break;
                            }

                            if current_line >= skip {
                                let left = removed_lines
                                    .get(j)
                                    .map(|line| {
                                        format_side_line(
                                            line,
                                            theme,
                                            true,
                                            max_width,
                                            horizontal_offset,
                                        )
                                    })
                                    .unwrap_or_else(|| Line::from(" ".repeat(max_width)));
                                let right = added_lines
                                    .get(j)
                                    .map(|line| {
                                        format_side_line(
                                            line,
                                            theme,
                                            false,
                                            max_width,
                                            horizontal_offset,
                                        )
                                    })
                                    .unwrap_or_else(|| Line::from(" ".repeat(max_width)));

                                left_lines.push(left);
                                right_lines.push(right);
                            }
                            current_line += 1;
                        }
                    }
                    LineType::Added => {
                        // Standalone added lines (not following removed)
                        if current_line >= skip {
                            let right_line = format_side_line(
                                hunk_line,
                                theme,
                                false,
                                max_width,
                                horizontal_offset,
                            );
                            // Pad empty left side to prevent artifacts
                            left_lines.push(Line::from(" ".repeat(max_width)));
                            right_lines.push(right_line);
                        }
                        current_line += 1;
                        i += 1;
                    }
                }
            }

            // Empty line between hunks (padded to prevent artifacts)
            if current_line >= skip && current_line < end_line {
                let empty = " ".repeat(max_width);
                left_lines.push(Line::from(empty.clone()));
                right_lines.push(Line::from(empty));
            }
            current_line += 1;
            if current_line >= end_line {
                break;
            }
        }
    }

    (left_lines, right_lines)
}

/// Format a line for side-by-side view with horizontal scrolling
fn format_side_line<'a>(
    hunk_line: &HunkLine,
    theme: &Theme,
    is_left: bool,
    max_width: usize,
    horizontal_offset: usize,
) -> Line<'a> {
    let (prefix, style) = match hunk_line.line_type {
        LineType::Added => ("+", theme.added_style()),
        LineType::Removed => ("-", theme.removed_style()),
        LineType::Context => (" ", theme.context_style()),
    };

    let line_num = if is_left {
        hunk_line
            .old_line_num
            .map(|n| format!("{n:4} "))
            .unwrap_or_else(|| "     ".to_string())
    } else {
        hunk_line
            .new_line_num
            .map(|n| format!("{n:4} "))
            .unwrap_or_else(|| "     ".to_string())
    };

    // Build full line (line number doesn't scroll, only content)
    let full_content = format!("{}{}", prefix, hunk_line.content);

    // Apply horizontal scroll to content only
    let scrolled_content = apply_horizontal_scroll(
        &full_content,
        horizontal_offset,
        max_width.saturating_sub(line_num.len()),
    );

    // Combine line number with scrolled content
    let mut display = format!("{line_num}{scrolled_content}");

    // Ensure the display string fits exactly within max_width by truncating or padding
    let display_len = display.chars().count();
    match display_len.cmp(&max_width) {
        std::cmp::Ordering::Greater => {
            // Truncate to max_width
            display = display.chars().take(max_width).collect();
        }
        std::cmp::Ordering::Less => {
            // Pad with spaces to max_width to prevent artifacts
            display.push_str(&" ".repeat(max_width - display_len));
        }
        std::cmp::Ordering::Equal => {}
    }

    Line::from(vec![Span::styled(display, style)])
}

/// Apply horizontal scroll with indicators
fn apply_horizontal_scroll(line: &str, offset: usize, max_width: usize) -> String {
    let chars: Vec<char> = line.chars().collect();

    if chars.is_empty() {
        return String::new();
    }

    let start_idx = offset.min(chars.len());
    let has_left = start_idx > 0;

    // Reserve space for indicators
    let available_width = if has_left {
        max_width.saturating_sub(1) // Space for '<'
    } else {
        max_width
    };

    let end_idx = (start_idx + available_width).min(chars.len());
    let has_right = end_idx < chars.len();

    // Adjust end if we need space for '>'
    let final_end = if has_right {
        end_idx.saturating_sub(1)
    } else {
        end_idx
    };

    let visible: String = chars[start_idx..final_end].iter().collect();

    format!(
        "{}{}{}",
        if has_left { "<" } else { "" },
        visible,
        if has_right { ">" } else { "" }
    )
}
