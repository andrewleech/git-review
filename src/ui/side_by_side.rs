use crate::app::App;
use crate::git::{HunkLine, LineType};
use crate::ui::theme::Theme;
use ratatui::text::{Line, Span};

/// Create side-by-side diff lines (left: old/removed, right: new/added)
/// Only creates lines within the visible window to save memory on large diffs
/// Aligns removed and added lines side-by-side for proper comparison
pub fn create_side_by_side_lines<'a>(
    app: &App,
    theme: &Theme,
    skip: usize,
    limit: usize,
) -> (Vec<Line<'a>>, Vec<Line<'a>>) {
    let mut left_lines = Vec::with_capacity(limit);
    let mut right_lines = Vec::with_capacity(limit);
    let mut current_line = 0;
    let end_line = skip + limit;

    if let Some(file) = app.selected_file() {
        // Show hunks
        for hunk in &file.hunks {
            // Hunk header on both sides
            if current_line >= skip && current_line < end_line {
                left_lines.push(Line::from(vec![Span::styled(
                    hunk.header.clone(),
                    theme.context_style(),
                )]));
                right_lines.push(Line::from(vec![Span::styled(
                    hunk.header.clone(),
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
                            let left_line = format_side_line(hunk_line, theme, true);
                            let right_line = format_side_line(hunk_line, theme, false);
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
                                let left = removed_lines.get(j).map(|line| format_side_line(line, theme, true))
                                    .unwrap_or_else(|| Line::from(""));
                                let right = added_lines.get(j).map(|line| format_side_line(line, theme, false))
                                    .unwrap_or_else(|| Line::from(""));

                                left_lines.push(left);
                                right_lines.push(right);
                            }
                            current_line += 1;
                        }
                    }
                    LineType::Added => {
                        // Standalone added lines (not following removed)
                        if current_line >= skip {
                            let right_line = format_side_line(hunk_line, theme, false);
                            left_lines.push(Line::from(""));
                            right_lines.push(right_line);
                        }
                        current_line += 1;
                        i += 1;
                    }
                }
            }

            // Empty line between hunks
            if current_line >= skip && current_line < end_line {
                left_lines.push(Line::from(""));
                right_lines.push(Line::from(""));
            }
            current_line += 1;
            if current_line >= end_line {
                break;
            }
        }
    }

    (left_lines, right_lines)
}

/// Format a line for side-by-side view
fn format_side_line<'a>(hunk_line: &HunkLine, theme: &Theme, is_left: bool) -> Line<'a> {
    let (prefix, style) = match hunk_line.line_type {
        LineType::Added => ("+", theme.added_style()),
        LineType::Removed => ("-", theme.removed_style()),
        LineType::Context => (" ", theme.context_style()),
    };

    let line_num = if is_left {
        hunk_line
            .old_line_num
            .map(|n| format!("{:4} ", n))
            .unwrap_or_else(|| "     ".to_string())
    } else {
        hunk_line
            .new_line_num
            .map(|n| format!("{:4} ", n))
            .unwrap_or_else(|| "     ".to_string())
    };

    let content = format!("{}{}{}", line_num, prefix, hunk_line.content);
    Line::from(vec![Span::styled(content, style)])
}
