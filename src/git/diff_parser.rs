use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineType {
    Context,
    Added,
    Removed,
}

#[derive(Debug, Clone)]
pub struct HunkLine {
    pub line_type: LineType,
    pub old_line_num: Option<usize>,
    pub new_line_num: Option<usize>,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct Hunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub header: String,
    pub lines: Vec<HunkLine>,
}

impl Hunk {
    /// Check if more context is available above this hunk
    pub fn can_expand_above(&self) -> bool {
        self.old_start > 1 || self.new_start > 1
    }

    /// Check if more context is available below this hunk
    pub fn can_expand_below(&self, file_lines: usize) -> bool {
        // Check if this hunk reaches the end of the file
        let hunk_end_line = self.new_start + self.new_lines;
        hunk_end_line < file_lines
    }

    /// Calculate how many lines can be expanded above
    pub fn available_lines_above(&self) -> usize {
        // Can expand up to the start of the file
        std::cmp::min(self.old_start.saturating_sub(1), self.new_start.saturating_sub(1))
    }
}

#[derive(Debug, Clone)]
pub struct FileDiff {
    pub old_path: String,
    pub new_path: String,
    pub hunks: Vec<Hunk>,
    pub new_file_lines: Option<usize>, // Total lines in new version (if known)
}

/// Parse a unified diff format into structured hunks
pub fn parse_diff(diff_text: &str) -> Result<Vec<FileDiff>> {
    let mut files = Vec::new();
    let mut current_file: Option<FileDiff> = None;
    let mut current_hunk: Option<Hunk> = None;

    let mut old_line_num = 0;
    let mut new_line_num = 0;

    for line in diff_text.lines() {
        if line.starts_with("diff --git") {
            // Save previous file if exists
            if let Some(mut file) = current_file.take() {
                if let Some(hunk) = current_hunk.take() {
                    file.hunks.push(hunk);
                }
                files.push(file);
            }

            // Start new file
            current_file = Some(FileDiff {
                old_path: String::new(),
                new_path: String::new(),
                hunks: Vec::new(),
                new_file_lines: None,
            });
        } else if line.starts_with("--- ") {
            if let Some(ref mut file) = current_file {
                file.old_path = line.strip_prefix("--- ").unwrap_or("").to_string();
                // Remove a/ or b/ prefix if present
                if file.old_path.starts_with("a/") {
                    file.old_path = file.old_path[2..].to_string();
                }
            }
        } else if line.starts_with("+++ ") {
            if let Some(ref mut file) = current_file {
                file.new_path = line.strip_prefix("+++ ").unwrap_or("").to_string();
                if file.new_path.starts_with("b/") {
                    file.new_path = file.new_path[2..].to_string();
                }
            }
        } else if line.starts_with("@@") {
            // Save previous hunk if exists
            if let Some(ref mut file) = current_file {
                if let Some(hunk) = current_hunk.take() {
                    file.hunks.push(hunk);
                }
            }

            // Parse hunk header: @@ -old_start,old_lines +new_start,new_lines @@
            let hunk_info = line
                .trim_start_matches("@@")
                .trim_end_matches("@@")
                .trim();

            if let Some((old_part, new_part)) = hunk_info.split_once(' ') {
                let old_parts: Vec<&str> = old_part.trim_start_matches('-').split(',').collect();
                let new_parts: Vec<&str> = new_part.trim_start_matches('+').split(',').collect();

                let old_start = old_parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
                let old_lines = old_parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
                let new_start = new_parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
                let new_lines = new_parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);

                old_line_num = old_start;
                new_line_num = new_start;

                current_hunk = Some(Hunk {
                    old_start,
                    old_lines,
                    new_start,
                    new_lines,
                    header: line.to_string(),
                    lines: Vec::new(),
                });
            }
        } else if let Some(ref mut hunk) = current_hunk {
            // Parse diff line
            if let Some(first_char) = line.chars().next() {
                let (line_type, content) = match first_char {
                    '+' => (LineType::Added, &line[1..]),
                    '-' => (LineType::Removed, &line[1..]),
                    ' ' => (LineType::Context, &line[1..]),
                    _ => continue,
                };

                let (old_num, new_num) = match line_type {
                    LineType::Context => {
                        let old = Some(old_line_num);
                        let new = Some(new_line_num);
                        old_line_num += 1;
                        new_line_num += 1;
                        (old, new)
                    }
                    LineType::Added => {
                        let new = Some(new_line_num);
                        new_line_num += 1;
                        (None, new)
                    }
                    LineType::Removed => {
                        let old = Some(old_line_num);
                        old_line_num += 1;
                        (old, None)
                    }
                };

                hunk.lines.push(HunkLine {
                    line_type,
                    old_line_num: old_num,
                    new_line_num: new_num,
                    content: content.to_string(),
                });
            }
        }
    }

    // Save final file and hunk
    if let Some(mut file) = current_file {
        if let Some(hunk) = current_hunk {
            file.hunks.push(hunk);
        }

        // Estimate file line count from last hunk
        if let Some(last_hunk) = file.hunks.last() {
            file.new_file_lines = Some(last_hunk.new_start + last_hunk.new_lines);
        }

        files.push(file);
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_diff() {
        let diff_text = r#"diff --git a/test.txt b/test.txt
--- a/test.txt
+++ b/test.txt
@@ -1,3 +1,3 @@
 line 1
-line 2
+line 2 modified
 line 3
"#;

        let files = parse_diff(diff_text).unwrap();
        assert_eq!(files.len(), 1);

        let file = &files[0];
        assert_eq!(file.old_path, "test.txt");
        assert_eq!(file.new_path, "test.txt");
        assert_eq!(file.hunks.len(), 1);

        let hunk = &file.hunks[0];
        assert_eq!(hunk.old_start, 1);
        assert_eq!(hunk.old_lines, 3);
        assert_eq!(hunk.new_start, 1);
        assert_eq!(hunk.new_lines, 3);
        assert_eq!(hunk.lines.len(), 4);

        assert_eq!(hunk.lines[0].line_type, LineType::Context);
        assert_eq!(hunk.lines[1].line_type, LineType::Removed);
        assert_eq!(hunk.lines[2].line_type, LineType::Added);
        assert_eq!(hunk.lines[3].line_type, LineType::Context);
    }

    #[test]
    fn test_hunk_expansion_available() {
        let hunk = Hunk {
            old_start: 10,
            old_lines: 5,
            new_start: 10,
            new_lines: 5,
            header: "@@ -10,5 +10,5 @@".to_string(),
            lines: Vec::new(),
        };

        assert!(hunk.can_expand_above());
        assert_eq!(hunk.available_lines_above(), 9);
    }
}
