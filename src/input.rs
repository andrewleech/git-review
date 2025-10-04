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
        (KeyCode::Char('s'), KeyModifiers::NONE) => {
            app.set_diff_mode(DiffMode::SideBySide);
        }
        (KeyCode::Char('i'), KeyModifiers::NONE) => {
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

        // Help (placeholder)
        (KeyCode::Char('?'), KeyModifiers::SHIFT) => {
            // TODO: Show help dialog
        }

        _ => {}
    }

    Ok(false)
}

/// Handle mouse input
pub fn handle_mouse_event(mouse: MouseEvent, app: &mut App) -> Result<()> {
    match mouse.kind {
        MouseEventKind::ScrollDown => {
            app.scroll(3);
        }
        MouseEventKind::ScrollUp => {
            app.scroll(-3);
        }
        MouseEventKind::Down(_button) => {
            // TODO: Handle clicks (commit selection, expand buttons, comments)
        }
        _ => {}
    }

    Ok(())
}
