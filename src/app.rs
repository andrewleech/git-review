use crate::config::{Config, DiffMode};
use crate::git::{CommitInfo, FileDiff};
use git2::Repository;

/// Application state
pub struct App {
    pub repo: Repository,
    pub commits: Vec<CommitInfo>,
    pub config: Config,

    // UI state
    pub selected_commit_index: usize,
    pub selected_file_index: usize,
    pub log_pane_visible: bool,
    pub help_visible: bool,
    pub scroll_offset: usize,
    pub horizontal_scroll: usize, // Horizontal scroll offset for side-by-side mode
    pub cursor_line: usize,       // Current line in diff view
    pub terminal_width: u16,
    pub terminal_height: u16,

    // Current diff data
    pub current_files: Vec<FileDiff>,
    pub current_context_lines: u32, // Context lines for current diff
}

impl App {
    pub fn new(repo: Repository, commits: Vec<CommitInfo>, config: Config) -> Self {
        // Get initial terminal size
        let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));

        let initial_context = config.display.context_lines;

        Self {
            repo,
            commits,
            config,
            selected_commit_index: 0,
            selected_file_index: 0,
            log_pane_visible: true,
            help_visible: false,
            scroll_offset: 0,
            horizontal_scroll: 0,
            cursor_line: 0,
            terminal_width: width,
            terminal_height: height,
            current_files: Vec::new(),
            current_context_lines: initial_context,
        }
    }

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

    /// Set diff mode
    pub fn set_diff_mode(&mut self, mode: DiffMode) {
        self.config.display.diff_mode = mode;
        // Reset horizontal scroll when switching modes
        self.reset_horizontal_scroll();
        // Save config - log error but don't fail
        if let Err(e) = self.config.save() {
            eprintln!("Warning: Failed to save config: {e}");
        }
    }

    /// Scroll diff view vertically with bounds checking
    pub fn scroll(&mut self, amount: isize) {
        let content_lines = self.calculate_content_lines();
        let visible_height = self.terminal_height.saturating_sub(3) as usize; // header + footer + borders
        let max_scroll = content_lines.saturating_sub(visible_height);

        if amount < 0 {
            self.scroll_offset = self.scroll_offset.saturating_sub(amount.unsigned_abs());
            self.cursor_line = self.cursor_line.saturating_sub(amount.unsigned_abs());
        } else {
            let new_offset = self.scroll_offset.saturating_add(amount as usize);
            self.scroll_offset = new_offset.min(max_scroll);
            self.cursor_line = self
                .cursor_line
                .saturating_add(amount as usize)
                .min(content_lines);
        }
    }

    /// Scroll diff view horizontally (side-by-side mode only)
    pub fn scroll_horizontal(&mut self, amount: isize) {
        if amount < 0 {
            // Scroll left
            self.horizontal_scroll = self.horizontal_scroll.saturating_sub(amount.unsigned_abs());
        } else {
            // Scroll right - no upper bound check, will be handled during rendering
            self.horizontal_scroll = self.horizontal_scroll.saturating_add(amount as usize);
        }
    }

    /// Reset horizontal scroll to start
    pub fn reset_horizontal_scroll(&mut self) {
        self.horizontal_scroll = 0;
    }

    /// Calculate total number of lines in current diff view
    fn calculate_content_lines(&self) -> usize {
        let mut total = 0;

        for (file_idx, file) in self.current_files.iter().enumerate() {
            // File separator (except first file)
            if file_idx > 0 {
                total += 3; // blank + separator + blank
            }

            // File header
            total += 3; // old path + new path + blank

            // Hunks
            for hunk in &file.hunks {
                total += 1; // hunk header
                total += hunk.lines.len();
                total += 1; // blank line after hunk

                // Account for expand buttons
                if hunk.available_lines_above() > 0 {
                    total += 1;
                }
                if let Some(file_lines) = file.new_file_lines {
                    if hunk.can_expand_below(file_lines) {
                        total += 1;
                    }
                }
            }
        }

        total
    }

    /// Handle terminal resize
    pub fn handle_resize(&mut self, width: u16, height: u16) {
        self.terminal_width = width;
        self.terminal_height = height;
    }

    /// Expand context for entire diff (git2 doesn't support per-hunk expansion)
    pub fn expand_context(&mut self) {
        let increment = self.config.display.context_expand_increment;
        self.current_context_lines += increment;
        self.load_diff_for_current_commit();
        // Reset scroll to show the expanded context
        self.scroll_offset = 0;
    }

    /// Reset context to default
    pub fn reset_context(&mut self) {
        self.current_context_lines = self.config.display.context_lines;
        self.load_diff_for_current_commit();
        self.scroll_offset = 0;
    }

    /// Load diff for currently selected commit
    fn load_diff_for_current_commit(&mut self) {
        if let Some(commit) = self.selected_commit() {
            // Generate diff with current context level
            let diff_options = crate::git::DiffOptions {
                context_lines: self.current_context_lines,
            };

            match crate::git::generate_diff(&self.repo, commit.id, &diff_options) {
                Ok(diff) => match crate::git::diff_to_text(&diff) {
                    Ok(text) => match crate::git::parse_diff(&text) {
                        Ok(files) => {
                            self.current_files = files;
                            self.selected_file_index = 0;
                        }
                        Err(e) => {
                            eprintln!("Failed to parse diff: {e}");
                            self.current_files = Vec::new();
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to convert diff to text: {e}");
                        self.current_files = Vec::new();
                    }
                },
                Err(e) => {
                    eprintln!("Failed to generate diff: {e}");
                    self.current_files = Vec::new();
                }
            }
        }
    }

    /// Initialize diff for first commit
    pub fn init_diff(&mut self) {
        self.load_diff_for_current_commit();
    }
}
