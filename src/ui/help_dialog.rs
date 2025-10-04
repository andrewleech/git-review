use crate::ui::theme::Theme;
use ratatui::{
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Render help dialog overlay
pub fn render(f: &mut Frame, area: Rect) {
    let theme = Theme::default();

    // Calculate dialog size (centered, 60x20)
    let dialog_width = area.width.min(60);
    let dialog_height = area.height.min(22);
    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    // Clear background
    f.render_widget(Clear, dialog_area);

    // Help content
    let help_text = vec![
        Line::from(Span::styled("Keyboard Shortcuts", theme.header_style())),
        Line::from(""),
        Line::from(vec![
            Span::styled("  q", theme.selected_style()),
            Span::raw("  - Quit application"),
        ]),
        Line::from(vec![
            Span::styled("  ?", theme.selected_style()),
            Span::raw("  - Show/hide this help dialog"),
        ]),
        Line::from(""),
        Line::from(Span::styled("Navigation", theme.header_style())),
        Line::from(vec![
            Span::styled("  j/k", theme.selected_style()),
            Span::raw(" or "),
            Span::styled("↓/↑", theme.selected_style()),
            Span::raw(" - Scroll diff view"),
        ]),
        Line::from(vec![
            Span::styled("  n/p", theme.selected_style()),
            Span::raw("  - Next/previous commit"),
        ]),
        Line::from(vec![
            Span::styled("  [/]", theme.selected_style()),
            Span::raw("  - Next/previous file"),
        ]),
        Line::from(""),
        Line::from(Span::styled("View Modes", theme.header_style())),
        Line::from(vec![
            Span::styled("  space", theme.selected_style()),
            Span::raw(" - Toggle commit log pane"),
        ]),
        Line::from(vec![
            Span::styled("  s", theme.selected_style()),
            Span::raw("  - Side-by-side diff mode"),
        ]),
        Line::from(vec![
            Span::styled("  i", theme.selected_style()),
            Span::raw("  - Inline diff mode"),
        ]),
        Line::from(""),
        Line::from(Span::styled("Mouse", theme.header_style())),
        Line::from("  Scroll wheel - Navigate diff"),
        Line::from(""),
        Line::from(Span::styled("Press ? or ESC to close", theme.context_style())),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(theme.selected_style()),
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    f.render_widget(help_paragraph, dialog_area);
}
