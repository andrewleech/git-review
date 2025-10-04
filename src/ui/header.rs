use crate::app::App;
use crate::ui::theme::Theme;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render the header bar
pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let theme = Theme::default();

    let header_text = if let Some(commit) = app.selected_commit() {
        if let Some(file) = app.selected_file() {
            format!(
                " {} | {} ({}/{}) | {}",
                commit.short_id,
                file.new_path,
                app.selected_file_index + 1,
                app.current_files.len(),
                commit.summary()
            )
        } else {
            format!(
                " {} | No files | {}",
                commit.short_id,
                commit.summary()
            )
        }
    } else {
        " No commit selected".to_string()
    };

    let header = Paragraph::new(header_text)
        .style(theme.header_style())
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(header, area);
}
