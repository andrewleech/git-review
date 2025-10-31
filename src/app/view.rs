use super::App;
use crate::comments::CommentLevel;
use crate::config::DiffMode;
use crate::git::LineType;

impl App {
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

    /// Scroll by full pages (visible height)
    pub fn scroll_page(&mut self, direction: isize) {
        let visible_height = self.terminal_height.saturating_sub(3) as usize;
        let amount = if direction < 0 {
            -(visible_height as isize)
        } else {
            visible_height as isize
        };
        self.scroll(amount);
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

    /// Detect what the cursor is pointing at for comment context
    /// Returns the comment level, line number, line type, and hunk header
    ///
    /// This function traverses the diff display to find where the cursor is positioned.
    /// It handles edge cases like expand buttons, file separators, and empty diffs.
    pub fn detect_comment_context(
        &self,
    ) -> (
        CommentLevel,
        Option<usize>,
        Option<LineType>,
        Option<String>,
    ) {
        // Edge case: no files means no valid comment context
        if self.current_files.is_empty() {
            return (CommentLevel::File, None, None, None);
        }

        // Calculate which line in the diff the cursor is on
        let mut current_line = 0;
        let target_line = self.cursor_line;

        for (file_idx, file) in self.current_files.iter().enumerate() {
            // File separator (except first file)
            if file_idx > 0 {
                current_line += 3; // blank + separator + blank
                if current_line > target_line {
                    // Cursor is on separator - treat as file-level for previous file
                    return (CommentLevel::File, None, None, None);
                }
            }

            // File header
            current_line += 3; // old path + new path + blank
            if current_line > target_line {
                // Cursor is on file header - file-level comment
                return (CommentLevel::File, None, None, None);
            }

            // Hunks
            for hunk in &file.hunks {
                // Hunk header
                let hunk_header_line = current_line;
                current_line += 1;

                // Expand button above if available
                if hunk.available_lines_above() > 0 {
                    current_line += 1;
                    // If cursor is on expand button, treat as hunk-level
                    if current_line - 1 == target_line {
                        return (
                            CommentLevel::Hunk,
                            None,
                            None,
                            Some(hunk.header.clone()),
                        );
                    }
                }

                // Hunk lines
                for line in hunk.lines.iter() {
                    if current_line == target_line {
                        // Cursor is on a specific line
                        let line_number = match line.line_type {
                            LineType::Added => line.new_line_num,
                            LineType::Removed => line.old_line_num,
                            LineType::Context => {
                                line.new_line_num.or(line.old_line_num)
                            }
                        };

                        if let Some(line_num) = line_number {
                            return (
                                CommentLevel::Line,
                                Some(line_num),
                                Some(line.line_type),
                                None,
                            );
                        }
                    }
                    current_line += 1;
                }

                // Expand button below if available
                if let Some(file_lines) = file.new_file_lines {
                    if hunk.can_expand_below(file_lines) {
                        current_line += 1;
                        // If cursor is on expand button, treat as hunk-level
                        if current_line - 1 == target_line {
                            return (
                                CommentLevel::Hunk,
                                None,
                                None,
                                Some(hunk.header.clone()),
                            );
                        }
                    }
                }

                // Blank line after hunk
                current_line += 1;

                // If cursor is within this hunk's range, return hunk context
                if target_line >= hunk_header_line && target_line < current_line {
                    return (
                        CommentLevel::Hunk,
                        None,
                        None,
                        Some(hunk.header.clone()),
                    );
                }
            }

        }

        // Default to file level (cursor beyond all content)
        (CommentLevel::File, None, None, None)
    }
}
