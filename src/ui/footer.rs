use crate::app::App;
use crate::ui::theme::Theme;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render the footer with keyboard shortcuts
pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let theme = Theme::default();

    // Show different shortcuts based on diff mode, with ?:help at the start for small screens
    let shortcuts = if app.config.display.diff_mode == crate::config::DiffMode::SideBySide {
        " ?:help | q:quit | /:search n:next N:prev | c:comment | v:view | space:log | s:side | i:inline | p/P:commit | Ctrl-PgUp/Dn:scroll"
    } else {
        " ?:help | q:quit | /:search n:next N:prev | c:comment | v:view | space:log | p/P:commit | e:expand | r:reset | Ctrl-PgUp/Dn:scroll"
    };

    let footer = Paragraph::new(shortcuts)
        .style(theme.header_style())
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(footer, area);
}
