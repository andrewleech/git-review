use crate::app::App;
use crate::git::LineType;
use crate::ui::theme::Theme;
use ratatui::text::Span;

/// Get comment count for a file
pub fn file_comment_count(app: &App, file_path: &str) -> usize {
    app.current_commit_comments()
        .map(|cc| cc.comments_for_file(file_path).len())
        .unwrap_or(0)
}

/// Create a comment indicator span for a file
pub fn file_indicator<'a>(app: &App, file_path: &str, theme: &Theme) -> Option<Span<'a>> {
    let count = file_comment_count(app, file_path);
    if count > 0 {
        Some(Span::styled(
            format!(" [{count}]"),
            theme.comment_indicator_style(),
        ))
    } else {
        None
    }
}

/// Get comment count for a line
pub fn line_comment_count(
    app: &App,
    file_path: &str,
    line_num: usize,
    line_type: LineType,
) -> usize {
    app.current_commit_comments()
        .map(|cc| cc.comments_at_line(file_path, line_num, line_type).len())
        .unwrap_or(0)
}

/// Create a comment indicator span for a line
pub fn line_indicator<'a>(
    app: &App,
    file_path: &str,
    line_num: usize,
    line_type: LineType,
    theme: &Theme,
) -> Option<Span<'a>> {
    let count = line_comment_count(app, file_path, line_num, line_type);
    if count > 0 {
        Some(Span::styled(
            format!(" [{count}]"),
            theme.comment_indicator_style(),
        ))
    } else {
        None
    }
}

/// Get comment count for a hunk
pub fn hunk_comment_count(app: &App, file_path: &str, hunk_header: &str) -> usize {
    app.current_commit_comments()
        .map(|cc| cc.comments_at_hunk(file_path, hunk_header).len())
        .unwrap_or(0)
}

/// Create a comment indicator span for a hunk
pub fn hunk_indicator<'a>(
    app: &App,
    file_path: &str,
    hunk_header: &str,
    theme: &Theme,
) -> Option<Span<'a>> {
    let count = hunk_comment_count(app, file_path, hunk_header);
    if count > 0 {
        Some(Span::styled(
            format!(" [{count}]"),
            theme.comment_indicator_style(),
        ))
    } else {
        None
    }
}
