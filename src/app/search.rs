use super::{App, SearchMatch, SearchMode};

impl App {
    /// Start search mode
    pub fn start_search(&mut self) {
        self.search_mode = SearchMode::Entering;
        self.search_query.clear();
    }

    /// Execute search with current query
    pub fn execute_search(&mut self) {
        self.search_matches.clear();
        self.current_match_index = None;

        if self.search_query.is_empty() {
            self.search_mode = SearchMode::Normal;
            return;
        }

        let query = self.search_query.to_lowercase();
        let mut line_index = 0;
        let mut matches = Vec::new();

        // Search through all files in current diff
        for (file_idx, file) in self.current_files.iter().enumerate() {
            // Add lines for file separator (except first file)
            if file_idx > 0 {
                line_index += 3; // blank + separator + blank
            }

            // File header lines (search in file paths)
            let old_path_lower = file.old_path.to_lowercase();
            let new_path_lower = file.new_path.to_lowercase();

            find_matches_in_line(&old_path_lower, line_index, &query, &mut matches);
            line_index += 1;
            find_matches_in_line(&new_path_lower, line_index, &query, &mut matches);
            line_index += 2; // new path + blank

            // Hunks
            for hunk in &file.hunks {
                // Search hunk header
                find_matches_in_line(&hunk.header.to_lowercase(), line_index, &query, &mut matches);
                line_index += 1;

                // Account for expand button above if available
                if hunk.available_lines_above() > 0 {
                    line_index += 1;
                }

                // Hunk lines
                for line in &hunk.lines {
                    let content_lower = line.content.to_lowercase();
                    find_matches_in_line(&content_lower, line_index, &query, &mut matches);
                    line_index += 1;
                }

                // Account for expand button below if available
                if let Some(file_lines) = file.new_file_lines {
                    if hunk.can_expand_below(file_lines) {
                        line_index += 1;
                    }
                }

                // Blank line after hunk
                line_index += 1;
            }
        }

        self.search_matches = matches;

        // Jump to first match if any found
        if !self.search_matches.is_empty() {
            self.current_match_index = Some(0);
            self.scroll_to_match(0);
            self.status_message = Some(format!(
                "Match 1/{} - Press n/N for next/prev",
                self.search_matches.len()
            ));
        } else {
            self.status_message = Some(format!("No matches found for '{}'", self.search_query));
        }

        self.search_mode = SearchMode::Normal;
    }

    /// Scroll to a specific match
    fn scroll_to_match(&mut self, match_idx: usize) {
        if let Some(search_match) = self.search_matches.get(match_idx) {
            // Set cursor to match line
            self.cursor_line = search_match.line_index;

            // Center the match in the viewport
            let visible_height = self.terminal_height.saturating_sub(3) as usize;
            let target_scroll = search_match
                .line_index
                .saturating_sub(visible_height / 2);

            let content_lines = self.calculate_content_lines_cached();
            let max_scroll = content_lines.saturating_sub(visible_height);

            self.scroll_offset = target_scroll.min(max_scroll);
        }
    }

    /// Move to next search match
    pub fn next_match(&mut self) {
        if self.search_matches.is_empty() {
            self.status_message = Some("No active search. Press / to search.".to_string());
            return;
        }

        let next_idx = match self.current_match_index {
            Some(idx) => {
                // Wrap around to first match
                if idx + 1 >= self.search_matches.len() {
                    0
                } else {
                    idx + 1
                }
            }
            None => 0,
        };

        self.current_match_index = Some(next_idx);
        self.scroll_to_match(next_idx);
        self.status_message = Some(format!(
            "Match {}/{}",
            next_idx + 1,
            self.search_matches.len()
        ));
    }

    /// Move to previous search match
    pub fn prev_match(&mut self) {
        if self.search_matches.is_empty() {
            self.status_message = Some("No active search. Press / to search.".to_string());
            return;
        }

        let prev_idx = match self.current_match_index {
            Some(idx) => {
                // Wrap around to last match
                if idx == 0 {
                    self.search_matches.len() - 1
                } else {
                    idx - 1
                }
            }
            None => 0,
        };

        self.current_match_index = Some(prev_idx);
        self.scroll_to_match(prev_idx);
        self.status_message = Some(format!(
            "Match {}/{}",
            prev_idx + 1,
            self.search_matches.len()
        ));
    }

    /// Clear search state
    pub fn clear_search(&mut self) {
        self.search_mode = SearchMode::Normal;
        self.search_query.clear();
        self.search_matches.clear();
        self.current_match_index = None;
        self.status_message = None;
    }

    /// Get matches for a specific line (for highlighting)
    pub fn get_matches_for_line(&self, line_index: usize) -> Vec<(usize, usize)> {
        self.search_matches
            .iter()
            .filter(|m| m.line_index == line_index)
            .map(|m| (m.char_start, m.char_end))
            .collect()
    }

    /// Calculate content lines (cached helper for search)
    fn calculate_content_lines_cached(&self) -> usize {
        // Reuse the existing calculate_content_lines from view module
        let mut total = 0;

        for (file_idx, file) in self.current_files.iter().enumerate() {
            if file_idx > 0 {
                total += 3;
            }
            total += 3;

            for hunk in &file.hunks {
                total += 1;
                total += hunk.lines.len();
                total += 1;

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
}

/// Find all matches in a single line (helper function)
fn find_matches_in_line(line_lower: &str, line_index: usize, query: &str, matches: &mut Vec<SearchMatch>) {
    let mut start = 0;

    while let Some(pos) = line_lower[start..].find(query) {
        let absolute_pos = start + pos;
        matches.push(SearchMatch {
            line_index,
            char_start: absolute_pos,
            char_end: absolute_pos + query.len(),
        });
        start = absolute_pos + 1; // Move past this match to find overlapping matches
    }
}
