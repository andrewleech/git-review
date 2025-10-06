use crate::app::App;
use crate::config::DiffMode;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

/// Handle keyboard input
///
/// Returns true if the app should exit
pub fn handle_key_event(key: KeyEvent, app: &mut App) -> Result<bool> {
    // When help is visible, only allow help toggle and ESC
    if app.help_visible {
        match (key.code, key.modifiers) {
            (KeyCode::Char('?'), _) | (KeyCode::Char('/'), KeyModifiers::SHIFT) => {
                app.toggle_help();
                return Ok(false);
            }
            (KeyCode::Esc, KeyModifiers::NONE) => {
                app.toggle_help();
                return Ok(false);
            }
            _ => return Ok(false), // Ignore all other keys when help is visible
        }
    }

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

        // Navigation - Vertical
        (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, KeyModifiers::NONE) => {
            app.scroll(1);
        }
        (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, KeyModifiers::NONE) => {
            app.scroll(-1);
        }

        // Navigation - Horizontal (side-by-side mode only)
        (KeyCode::Char('h'), KeyModifiers::NONE) | (KeyCode::Left, KeyModifiers::NONE) => {
            if app.config.display.diff_mode == DiffMode::SideBySide {
                let amount = app.config.display.horizontal_scroll_amount as isize;
                app.scroll_horizontal(-amount);
            }
        }
        (KeyCode::Char('l'), KeyModifiers::NONE) | (KeyCode::Right, KeyModifiers::NONE) => {
            if app.config.display.diff_mode == DiffMode::SideBySide {
                let amount = app.config.display.horizontal_scroll_amount as isize;
                app.scroll_horizontal(amount);
            }
        }

        // Commit navigation
        (KeyCode::Char('n'), KeyModifiers::NONE) => {
            app.next_commit();
        }
        (KeyCode::Char('p'), KeyModifiers::NONE) => {
            app.previous_commit();
        }

        // File navigation
        (KeyCode::PageUp, KeyModifiers::NONE) => {
            app.previous_file();
        }
        (KeyCode::PageDown, KeyModifiers::NONE) => {
            app.next_file();
        }

        // Context expansion (expands entire diff since git2 doesn't support per-hunk)
        (KeyCode::Char('e'), KeyModifiers::NONE) | (KeyCode::Char('E'), KeyModifiers::SHIFT) => {
            app.expand_context();
        }
        // Reset context to default
        (KeyCode::Char('r'), KeyModifiers::NONE) | (KeyCode::Char('R'), KeyModifiers::SHIFT) => {
            app.reset_context();
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
            // ESC only handled here when help is not visible (would be caught above otherwise)
        }

        _ => {}
    }

    Ok(false)
}

/// Handle mouse input
pub fn handle_mouse_event(mouse: MouseEvent, app: &mut App) -> Result<()> {
    // Ignore all mouse events when help is visible
    if app.help_visible {
        return Ok(());
    }

    match mouse.kind {
        MouseEventKind::ScrollDown => {
            app.scroll(3);
        }
        MouseEventKind::ScrollUp => {
            app.scroll(-3);
        }
        MouseEventKind::Down(_button) => {
            handle_mouse_click(mouse, app)?;
        }
        _ => {}
    }

    Ok(())
}

/// Handle mouse click events
fn handle_mouse_click(mouse: MouseEvent, app: &mut App) -> Result<()> {
    // Use stored terminal size to avoid race condition on resize
    let width = app.terminal_width;
    let height = app.terminal_height;

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
