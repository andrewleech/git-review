use crate::app::{App, SearchMode};
use crate::ui::theme::Theme;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render search prompt at bottom of screen (like less)
pub fn render(f: &mut Frame, app: &App, area: Rect) {
    // Only render if in search mode
    if app.search_mode != SearchMode::Entering {
        return;
    }

    let theme = Theme::default();

    // Calculate prompt area at bottom of screen
    let prompt_height = 3; // 1 line + borders
    let prompt_area = Rect::new(
        area.x,
        area.y + area.height.saturating_sub(prompt_height),
        area.width,
        prompt_height,
    );

    // Build prompt text with cursor
    let prompt_text = format!("/{}_", app.search_query);

    let prompt_paragraph = Paragraph::new(Line::from(vec![Span::styled(
        prompt_text,
        theme.selected_style(),
    )]))
    .block(
        Block::default()
            .title(" Search ")
            .borders(Borders::ALL)
            .border_style(theme.selected_style()),
    );

    f.render_widget(prompt_paragraph, prompt_area);
}
