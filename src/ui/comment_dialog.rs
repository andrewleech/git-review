use crate::app::{App, CommentMode};
use crate::comments::CommentLevel;
use crate::ui::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Handle keyboard input in comment dialog
pub fn handle_key(key: KeyEvent, app: &mut App) -> anyhow::Result<bool> {
    match (key.code, key.modifiers) {
        // Save comment
        (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
            app.save_comment()?;
            return Ok(false);
        }
        // Cancel
        (KeyCode::Esc, KeyModifiers::NONE) => {
            app.cancel_comment();
            return Ok(false);
        }
        // Text input
        (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
            app.comment_draft.push(c);
        }
        // Backspace
        (KeyCode::Backspace, KeyModifiers::NONE) => {
            app.comment_draft.pop();
        }
        // Enter (newline)
        (KeyCode::Enter, KeyModifiers::NONE) => {
            app.comment_draft.push('\n');
        }
        _ => {}
    }
    Ok(false)
}

/// Render comment creation dialog
pub fn render_create(f: &mut Frame, app: &App, area: Rect) {
    if let CommentMode::Creating {
        level, file_path, ..
    } = &app.comment_mode
    {
        let theme = Theme::default();

        // Calculate dialog size (centered, 60% width, 40% height, min 40x10)
        let dialog_width = (area.width * 60 / 100).max(40).min(area.width);
        let dialog_height = (area.height * 40 / 100).max(10).min(area.height);
        let x = (area.width.saturating_sub(dialog_width)) / 2;
        let y = (area.height.saturating_sub(dialog_height)) / 2;

        let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

        // Clear background
        f.render_widget(Clear, dialog_area);

        // Build title
        let level_str = match level {
            CommentLevel::Line => "Line",
            CommentLevel::Hunk => "Hunk",
            CommentLevel::File => "File",
        };
        let title = format!(" New {level_str} Comment: {file_path} ");

        // Build content lines
        let mut lines = vec![
            Line::from(Span::styled(
                "Type your comment below:",
                theme.context_style(),
            )),
            Line::from(""),
        ];

        // Add comment text (with cursor)
        for line_text in app.comment_draft.lines() {
            lines.push(Line::from(line_text.to_string()));
        }

        // Add cursor indicator
        lines.push(Line::from(Span::styled("_", theme.selected_style())));
        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Ctrl+S", theme.selected_style()),
            Span::raw(" save  |  "),
            Span::styled("ESC", theme.selected_style()),
            Span::raw(" cancel"),
        ]));

        let dialog = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(theme.selected_style()),
            )
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });

        f.render_widget(dialog, dialog_area);
    }
}

/// Render comment viewing dialog
pub fn render_view(f: &mut Frame, app: &App, area: Rect) {
    if let CommentMode::ViewingComments(comments) = &app.comment_mode {
        let theme = Theme::default();

        // Calculate dialog size
        let dialog_width = (area.width * 70 / 100).max(50).min(area.width);
        let dialog_height = (area.height * 60 / 100).max(15).min(area.height);
        let x = (area.width.saturating_sub(dialog_width)) / 2;
        let y = (area.height.saturating_sub(dialog_height)) / 2;

        let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

        f.render_widget(Clear, dialog_area);

        let mut lines = vec![Line::from(Span::styled(
            format!("Comments ({}):", comments.len()),
            theme.header_style(),
        ))];

        for (i, comment) in comments.iter().enumerate() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled(format!("[{}] ", i + 1), theme.selected_style()),
                Span::styled(comment.location_desc(), theme.context_style()),
            ]));
            lines.push(Line::from(comment.text.clone()));
            lines.push(Line::from(Span::styled(
                format!("  -- {}", comment.created_at.format("%Y-%m-%d %H:%M")),
                theme.context_style(),
            )));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("ESC", theme.selected_style()),
            Span::raw(" close"),
        ]));

        let dialog = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Comments ")
                    .borders(Borders::ALL)
                    .border_style(theme.selected_style()),
            )
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });

        f.render_widget(dialog, dialog_area);
    }
}
