use super::App;
use crate::git::{CommitInfo, FileDiff};

impl App {
    /// Get currently selected commit
    pub fn selected_commit(&self) -> Option<&CommitInfo> {
        self.commits.get(self.selected_commit_index)
    }

    /// Get currently selected file
    pub fn selected_file(&self) -> Option<&FileDiff> {
        self.current_files.get(self.selected_file_index)
    }

    /// Select a specific commit by index
    pub fn select_commit(&mut self, index: usize) {
        if index < self.commits.len() && index != self.selected_commit_index {
            self.selected_commit_index = index;
            self.scroll_offset = 0;
            self.reset_horizontal_scroll();
            self.reset_context(); // Reset to default context for new commit
        }
    }

    /// Navigate to next commit
    pub fn next_commit(&mut self) {
        if self.selected_commit_index + 1 < self.commits.len() {
            self.selected_commit_index += 1;
            self.scroll_offset = 0;
            self.reset_horizontal_scroll();
            self.reset_context(); // Reset to default context for new commit
        }
    }

    /// Navigate to previous commit
    pub fn previous_commit(&mut self) {
        if self.selected_commit_index > 0 {
            self.selected_commit_index -= 1;
            self.scroll_offset = 0;
            self.reset_horizontal_scroll();
            self.reset_context(); // Reset to default context for new commit
        }
    }

    /// Navigate to next file in current diff
    pub fn next_file(&mut self) {
        if self.selected_file_index + 1 < self.current_files.len() {
            self.selected_file_index += 1;
            self.scroll_offset = 0;
            self.reset_horizontal_scroll();
        }
    }

    /// Navigate to previous file in current diff
    pub fn previous_file(&mut self) {
        if self.selected_file_index > 0 {
            self.selected_file_index -= 1;
            self.scroll_offset = 0;
            self.reset_horizontal_scroll();
        }
    }

    /// Toggle log pane visibility
    pub fn toggle_log_pane(&mut self) {
        self.log_pane_visible = !self.log_pane_visible;
    }

    /// Toggle help dialog visibility
    pub fn toggle_help(&mut self) {
        self.help_visible = !self.help_visible;
    }
}
