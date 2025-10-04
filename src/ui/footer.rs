use crate::app::App;
use crate::ui::theme::Theme;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render the footer with keyboard shortcuts
pub fn render(f: &mut Frame, _app: &App, area: Rect) {
    let theme = Theme::default();

    let shortcuts = " q:quit | space:log | s:side-by-side | i:inline | [ ]:files | n/p:commit | e:expand | r:reset | ?:help";

    let footer = Paragraph::new(shortcuts)
        .style(theme.header_style())
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(footer, area);
}
