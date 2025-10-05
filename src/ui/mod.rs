pub mod diff_view;
pub mod footer;
pub mod header;
pub mod help_dialog;
pub mod hunk_expander;
pub mod layout;
pub mod log_pane;
pub mod side_by_side;
pub mod theme;

use crate::app::App;
use anyhow::Result;
use ratatui::Frame;

/// Main render function
pub fn render(f: &mut Frame, app: &App) -> Result<()> {
    let size = f.area();

    // Calculate layout
    let layout_info = layout::calculate_layout(
        size.width,
        size.height,
        app.log_pane_visible,
        app.config.ui.log_pane_width_ratio,
    );

    // Render header
    if let Some(header_area) = layout_info.header {
        header::render(f, app, header_area);
    }

    // Render main content area
    if let Some(content_area) = layout_info.content {
        if app.log_pane_visible {
            // Split content into log pane and diff view
            if let (Some(log_area), Some(diff_area)) =
                (layout_info.log_pane, layout_info.diff_area)
            {
                log_pane::render(f, app, log_area);
                diff_view::render(f, app, diff_area)?;
            }
        } else {
            // Full width diff view
            diff_view::render(f, app, content_area)?;
        }
    }

    // Render footer
    if let Some(footer_area) = layout_info.footer {
        footer::render(f, app, footer_area);
    }

    // Render help dialog on top if visible
    if app.help_visible {
        help_dialog::render(f, size);
    }

    Ok(())
}
