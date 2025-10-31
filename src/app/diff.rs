use super::App;

impl App {
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
