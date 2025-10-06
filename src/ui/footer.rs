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

    // Show different shortcuts based on diff mode
    let shortcuts = if app.config.display.diff_mode == crate::config::DiffMode::SideBySide {
        " q:quit | space:log | s:side-by-side | i:inline | h/l:scroll-horiz | n/p:commit | PgUp/Dn:files | ?:help"
    } else {
        " q:quit | space:log | s:side-by-side | i:inline | n/p:commit | PgUp/Dn:files | e:expand | r:reset | ?:help"
    };

    let footer = Paragraph::new(shortcuts)
        .style(theme.header_style())
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(footer, area);
}
