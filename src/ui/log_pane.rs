use crate::app::App;
use crate::ui::theme::Theme;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

/// Render the commit log pane
pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let theme = Theme::default();

    // Create list items from commits
    let items: Vec<ListItem> = app
        .commits
        .iter()
        .enumerate()
        .map(|(idx, commit)| {
            let is_selected = idx == app.selected_commit_index;

            // Format: [hash] message
            let content = format!("[{}] {}", commit.short_id, commit.summary());

            // Truncate to fit area width
            let max_width = area.width.saturating_sub(4) as usize; // Account for borders
            let display_content = if content.len() > max_width {
                format!("{}...", &content[..max_width.saturating_sub(3)])
            } else {
                content
            };

            let style = if is_selected {
                theme.selected_style()
            } else {
                theme.normal_style()
            };

            let line = Line::from(vec![Span::styled(display_content, style)]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Commits ")
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        )
        .highlight_style(theme.selected_style());

    f.render_widget(list, area);
}
