use crate::app::App;
use crate::config::DiffMode;
use crate::git::{HunkLine, LineType};
use crate::ui::theme::Theme;
use anyhow::Result;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Render the diff view
pub fn render(f: &mut Frame, app: &App, area: Rect) -> Result<()> {
    let theme = Theme::default();

    if app.current_files.is_empty() {
        // No diff to display
        let placeholder = Paragraph::new("No diff available. Loading...")
            .block(
                Block::default()
                    .title(" Diff ")
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(placeholder, area);
        return Ok(());
    }

    match app.config.display.diff_mode {
        DiffMode::SideBySide => render_side_by_side(f, app, area, &theme),
        DiffMode::Inline => render_inline(f, app, area, &theme),
    }

    Ok(())
}

/// Render side-by-side diff mode
fn render_side_by_side(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    use ratatui::layout::{Constraint, Direction, Layout};

    if let Some(file) = app.selected_file() {
        // Split area into left (removed) and right (added) columns
        let content_area = area;
        let half_width = content_area.width / 2;

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(half_width),
                Constraint::Length(content_area.width.saturating_sub(half_width)),
            ])
            .split(content_area);

        let left_area = columns[0];
        let right_area = columns[1];

        let visible_lines_count = left_area.height.saturating_sub(2) as usize;
        // Account for borders when calculating content width
        let max_width = left_area.width.saturating_sub(2) as usize;

        // Create only visible lines for side-by-side view to save memory
        let (left_lines, right_lines) = crate::ui::side_by_side::create_side_by_side_lines(
            app,
            theme,
            app.scroll_offset,
            visible_lines_count,
            max_width,
            app.horizontal_scroll,
        );

        let left_paragraph = Paragraph::new(left_lines).block(
            Block::default()
                .title(format!(" Old: {} ", file.old_path))
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        );

        // Render right side (added lines)
        let right_paragraph = Paragraph::new(right_lines).block(
            Block::default()
                .title(format!(" New: {} ", file.new_path))
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        );

        f.render_widget(left_paragraph, left_area);
        f.render_widget(right_paragraph, right_area);
    } else {
        // No file selected
        let placeholder = Paragraph::new("No file selected").block(
            Block::default()
                .title(" Diff (Side-by-Side) ")
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        );
        f.render_widget(placeholder, area);
    }
}

/// Render inline diff mode
fn render_inline(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let lines = create_diff_lines(app, theme);
    let visible_lines = lines
        .into_iter()
        .skip(app.scroll_offset)
        .take(area.height.saturating_sub(2) as usize)
        .collect::<Vec<_>>();

    let title = if app.current_files.len() > 1 {
        format!(" Diff (Inline) - {} files ", app.current_files.len())
    } else {
        " Diff (Inline) ".to_string()
    };

    let diff_paragraph = Paragraph::new(visible_lines)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(diff_paragraph, area);
}

/// Create styled lines from all files' diffs
fn create_diff_lines<'a>(app: &App, theme: &Theme) -> Vec<Line<'a>> {
    let mut lines = Vec::new();

    // Show all files in one scrollable view
    for (file_idx, file) in app.current_files.iter().enumerate() {
        // File separator (except before first file)
        if file_idx > 0 {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "─".repeat(80),
                theme.context_style(),
            )]));
            lines.push(Line::from(""));
        }

        // Show file header
        lines.push(Line::from(vec![Span::styled(
            format!("--- {}", file.old_path),
            theme.removed_style(),
        )]));
        lines.push(Line::from(vec![Span::styled(
            format!("+++ {}", file.new_path),
            theme.added_style(),
        )]));
        lines.push(Line::from(""));

        // Show hunks with expand buttons
        for (hunk_idx, hunk) in file.hunks.iter().enumerate() {
            let hunk_id = crate::app::HunkId {
                file_path: file.new_path.clone(),
                hunk_index: hunk_idx,
            };

            // Expand button above (if more context available)
            if let Some(expand_line) = crate::ui::hunk_expander::create_expand_above_line(
                hunk,
                theme,
                &hunk_id,
                app.config.display.context_expand_increment,
            ) {
                lines.push(expand_line);
            }

            // Hunk header
            lines.push(Line::from(vec![Span::styled(
                hunk.header.clone(),
                theme.context_style(),
            )]));

            // Hunk lines
            for hunk_line in &hunk.lines {
                let line = format_hunk_line(hunk_line, theme);
                lines.push(line);
            }

            // Expand button below (with file length check)
            if let Some(expand_line) = crate::ui::hunk_expander::create_expand_below_line(
                hunk,
                theme,
                &hunk_id,
                app.config.display.context_expand_increment,
                file.new_file_lines,
            ) {
                lines.push(expand_line);
            }

            // Empty line between hunks
            lines.push(Line::from(""));
        }
    }

    lines
}

/// Format a single hunk line with appropriate styling
fn format_hunk_line<'a>(hunk_line: &HunkLine, theme: &Theme) -> Line<'a> {
    let (prefix, style) = match hunk_line.line_type {
        LineType::Added => ("+", theme.added_style()),
        LineType::Removed => ("-", theme.removed_style()),
        LineType::Context => (" ", theme.context_style()),
    };

    let line_num = match hunk_line.line_type {
        LineType::Added => hunk_line
            .new_line_num
            .map(|n| format!("{:4} ", n))
            .unwrap_or_else(|| "     ".to_string()),
        LineType::Removed => hunk_line
            .old_line_num
            .map(|n| format!("{:4} ", n))
            .unwrap_or_else(|| "     ".to_string()),
        LineType::Context => hunk_line
            .old_line_num
            .or(hunk_line.new_line_num)
            .map(|n| format!("{:4} ", n))
            .unwrap_or_else(|| "     ".to_string()),
    };

    let content = format!("{}{}{}", line_num, prefix, hunk_line.content);

    Line::from(vec![Span::styled(content, style)])
}
