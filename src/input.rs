use crate::app::App;
use crate::config::DiffMode;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

/// Handle keyboard input
///
/// Returns true if the app should exit
pub fn handle_key_event(key: KeyEvent, app: &mut App) -> Result<bool> {
    match (key.code, key.modifiers) {
        // Quit
        (KeyCode::Char('q'), KeyModifiers::NONE) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
            return Ok(true);
        }

        // Toggle log pane
        (KeyCode::Char(' '), KeyModifiers::NONE) => {
            app.toggle_log_pane();
        }

        // Diff mode switching
        (KeyCode::Char('s'), KeyModifiers::NONE) | (KeyCode::Char('S'), KeyModifiers::SHIFT) => {
            app.set_diff_mode(DiffMode::SideBySide);
        }
        (KeyCode::Char('i'), KeyModifiers::NONE) | (KeyCode::Char('I'), KeyModifiers::SHIFT) => {
            app.set_diff_mode(DiffMode::Inline);
        }

        // Navigation
        (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, KeyModifiers::NONE) => {
            app.scroll(1);
        }
        (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, KeyModifiers::NONE) => {
            app.scroll(-1);
        }

        // Commit navigation
        (KeyCode::Char('n'), KeyModifiers::NONE) => {
            app.next_commit();
        }
        (KeyCode::Char('p'), KeyModifiers::NONE) => {
            app.previous_commit();
        }

        // File navigation
        (KeyCode::Char('['), KeyModifiers::NONE) => {
            app.previous_file();
        }
        (KeyCode::Char(']'), KeyModifiers::NONE) => {
            app.next_file();
        }

        // Context expansion (placeholder - needs cursor position)
        (KeyCode::Char('e'), KeyModifiers::NONE) => {
            // TODO: Expand context below current line
        }
        (KeyCode::Char('E'), KeyModifiers::SHIFT) => {
            // TODO: Expand context above current line
        }

        // Comments (placeholder for future implementation)
        (KeyCode::Char('c'), KeyModifiers::NONE) => {
            // TODO: Add/edit comment on current line
        }
        (KeyCode::Char('v'), KeyModifiers::NONE) => {
            // TODO: View comments on current line
        }
        (KeyCode::Char('d'), KeyModifiers::NONE) => {
            // TODO: Delete comment
        }

        // Help - handle both ? directly and Shift+/
        (KeyCode::Char('?'), _) | (KeyCode::Char('/'), KeyModifiers::SHIFT) => {
            app.toggle_help();
        }
        (KeyCode::Esc, KeyModifiers::NONE) => {
            if app.help_visible {
                app.toggle_help();
            }
        }

        _ => {}
    }

    Ok(false)
}

/// Handle mouse input
pub fn handle_mouse_event(mouse: MouseEvent, app: &mut App, terminal_size: (u16, u16)) -> Result<()> {
    match mouse.kind {
        MouseEventKind::ScrollDown => {
            app.scroll(3);
        }
        MouseEventKind::ScrollUp => {
            app.scroll(-3);
        }
        MouseEventKind::Down(_button) => {
            handle_mouse_click(mouse, app, terminal_size)?;
        }
        _ => {}
    }

    Ok(())
}

/// Handle mouse click events
fn handle_mouse_click(mouse: MouseEvent, app: &mut App, terminal_size: (u16, u16)) -> Result<()> {
    let (width, height) = terminal_size;

    // Calculate layout to determine clickable regions
    let layout_info = crate::ui::layout::calculate_layout(
        width,
        height,
        app.log_pane_visible,
        app.config.ui.log_pane_width_ratio,
    );

    // Check if click is in log pane
    if app.log_pane_visible {
        if let Some(log_area) = layout_info.log_pane {
            if mouse.column >= log_area.x
                && mouse.column < log_area.x + log_area.width
                && mouse.row >= log_area.y
                && mouse.row < log_area.y + log_area.height
            {
                // Click is in log pane - calculate which commit
                // Account for borders (top border = 1 line) and title
                let relative_row = mouse.row.saturating_sub(log_area.y + 1);

                if relative_row < app.commits.len() as u16 {
                    app.select_commit(relative_row as usize);
                }
                return Ok(());
            }
        }
    }

    // TODO: Handle clicks in diff view for:
    // - Expand button clicks (need to track which lines are expand buttons)
    // - Comment indicators (when implemented)

    Ok(())
}
