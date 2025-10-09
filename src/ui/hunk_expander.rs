use crate::git::Hunk;
use crate::ui::theme::Theme;
use ratatui::text::{Line, Span};

/// Create an expand button line for above a hunk
pub fn create_expand_above_line<'a>(
    hunk: &Hunk,
    theme: &Theme,
    increment: u32,
) -> Option<Line<'a>> {
    let available = hunk.available_lines_above();
    if available == 0 {
        return None;
    }

    let lines_to_show = available.min(increment as usize);
    let text = format!("  ↑ Expand {lines_to_show} more lines ↑  ");

    Some(Line::from(vec![Span::styled(text, theme.context_style())]))
}

/// Create an expand button line for below a hunk
pub fn create_expand_below_line<'a>(
    hunk: &Hunk,
    theme: &Theme,
    increment: u32,
    file_lines: Option<usize>,
) -> Option<Line<'a>> {
    // Check if more context is available below
    if let Some(total_lines) = file_lines {
        if !hunk.can_expand_below(total_lines) {
            return None;
        }
    }

    let lines_to_show = increment as usize;
    let text = format!("  ↓ Expand {lines_to_show} more lines ↓  ");

    Some(Line::from(vec![Span::styled(text, theme.context_style())]))
}
