use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::git::diff_parser::LineType;

/// Level of granularity for a comment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommentLevel {
    /// Comment on a specific line
    Line,
    /// Comment on an entire hunk
    Hunk,
    /// Comment on an entire file
    File,
}

/// Location within a file where a comment is attached
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum CommentLocation {
    /// Specific line in the diff
    Line {
        /// Line number (old or new depending on line_type)
        number: usize,
        /// Whether this is an added, removed, or context line
        #[serde(rename = "line_type")]
        kind: LineType,
    },
    /// Entire hunk identified by its header
    Hunk {
        /// The hunk header string (e.g., "@@ -40,5 +40,6 @@")
        header: String,
    },
    /// File-level comment (no specific location)
    File,
}

/// A single comment attached to code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// Granularity level of this comment
    pub level: CommentLevel,
    /// Path to the file this comment is about
    pub file_path: String,
    /// Specific location within the file
    pub location: CommentLocation,
    /// The comment text content
    pub text: String,
    /// When this comment was created
    pub created_at: DateTime<Local>,
}

impl Comment {
    /// Create a new line-level comment
    pub fn new_line(
        file_path: String,
        line_number: usize,
        line_type: LineType,
        text: String,
    ) -> Self {
        Self {
            level: CommentLevel::Line,
            file_path,
            location: CommentLocation::Line {
                number: line_number,
                kind: line_type,
            },
            text,
            created_at: Local::now(),
        }
    }

    /// Create a new hunk-level comment
    pub fn new_hunk(file_path: String, hunk_header: String, text: String) -> Self {
        Self {
            level: CommentLevel::Hunk,
            file_path,
            location: CommentLocation::Hunk {
                header: hunk_header,
            },
            text,
            created_at: Local::now(),
        }
    }

    /// Create a new file-level comment
    pub fn new_file(file_path: String, text: String) -> Self {
        Self {
            level: CommentLevel::File,
            file_path,
            location: CommentLocation::File,
            text,
            created_at: Local::now(),
        }
    }

    /// Check if this comment matches a specific line location
    pub fn matches_line(&self, file: &str, line_num: usize, line_type: LineType) -> bool {
        if self.file_path != file {
            return false;
        }
        match &self.location {
            CommentLocation::Line { number, kind } => *number == line_num && *kind == line_type,
            _ => false,
        }
    }

    /// Check if this comment matches a specific hunk
    pub fn matches_hunk(&self, file: &str, hunk_header: &str) -> bool {
        if self.file_path != file {
            return false;
        }
        match &self.location {
            CommentLocation::Hunk { header } => header == hunk_header,
            _ => false,
        }
    }

    /// Check if this comment is a file-level comment for the given file
    pub fn matches_file(&self, file: &str) -> bool {
        self.file_path == file && matches!(self.location, CommentLocation::File)
    }

    /// Get a short description of the location for display
    pub fn location_desc(&self) -> String {
        match &self.location {
            CommentLocation::Line { number, kind } => {
                let kind_str = match kind {
                    LineType::Added => "added",
                    LineType::Removed => "removed",
                    LineType::Context => "context",
                };
                format!("Line {} ({})", number, kind_str)
            }
            CommentLocation::Hunk { header } => format!("Hunk: {}", header),
            CommentLocation::File => "File-level".to_string(),
        }
    }
}

/// Collection of comments for a single commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitComments {
    /// The commit ID (full hash)
    pub commit_id: String,
    /// The branch this review was performed on
    pub branch: String,
    /// When this comment collection was last updated
    pub timestamp: DateTime<Local>,
    /// All comments for this commit
    pub comments: Vec<Comment>,
}

impl CommitComments {
    /// Create a new comment collection for a commit
    pub fn new(commit_id: String, branch: String) -> Self {
        Self {
            commit_id,
            branch,
            timestamp: Local::now(),
            comments: Vec::new(),
        }
    }

    /// Add a comment to this collection
    pub fn add_comment(&mut self, comment: Comment) {
        self.comments.push(comment);
        self.timestamp = Local::now();
    }

    /// Remove a comment by index
    pub fn remove_comment(&mut self, index: usize) -> Option<Comment> {
        if index < self.comments.len() {
            self.timestamp = Local::now();
            Some(self.comments.remove(index))
        } else {
            None
        }
    }

    /// Get all comments for a specific file
    pub fn comments_for_file(&self, file_path: &str) -> Vec<&Comment> {
        self.comments
            .iter()
            .filter(|c| c.file_path == file_path)
            .collect()
    }

    /// Get all line-level comments for a specific location
    pub fn comments_at_line(
        &self,
        file: &str,
        line_num: usize,
        line_type: LineType,
    ) -> Vec<&Comment> {
        self.comments
            .iter()
            .filter(|c| c.matches_line(file, line_num, line_type))
            .collect()
    }

    /// Get all hunk-level comments for a specific hunk
    pub fn comments_at_hunk(&self, file: &str, hunk_header: &str) -> Vec<&Comment> {
        self.comments
            .iter()
            .filter(|c| c.matches_hunk(file, hunk_header))
            .collect()
    }

    /// Get file-level comments for a specific file
    pub fn file_level_comments(&self, file: &str) -> Vec<&Comment> {
        self.comments
            .iter()
            .filter(|c| c.matches_file(file))
            .collect()
    }

    /// Serialize to JSON string for storage
    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    /// Check if there are any comments
    pub fn is_empty(&self) -> bool {
        self.comments.is_empty()
    }

    /// Get the total number of comments
    pub fn len(&self) -> usize {
        self.comments.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_creation() {
        let comment =
            Comment::new_line("src/main.rs".to_string(), 42, LineType::Added, "Test".to_string());

        assert_eq!(comment.level, CommentLevel::Line);
        assert_eq!(comment.file_path, "src/main.rs");
        assert_eq!(comment.text, "Test");
    }

    #[test]
    fn test_comment_matching() {
        let comment =
            Comment::new_line("src/main.rs".to_string(), 42, LineType::Added, "Test".to_string());

        assert!(comment.matches_line("src/main.rs", 42, LineType::Added));
        assert!(!comment.matches_line("src/main.rs", 43, LineType::Added));
        assert!(!comment.matches_line("src/main.rs", 42, LineType::Removed));
        assert!(!comment.matches_line("src/other.rs", 42, LineType::Added));
    }

    #[test]
    fn test_commit_comments_serialization() {
        let mut commit_comments = CommitComments::new("abc123".to_string(), "main".to_string());

        commit_comments.add_comment(Comment::new_line(
            "src/main.rs".to_string(),
            42,
            LineType::Added,
            "Test comment".to_string(),
        ));

        let json = commit_comments.to_json().unwrap();
        let deserialized = CommitComments::from_json(&json).unwrap();

        assert_eq!(deserialized.commit_id, "abc123");
        assert_eq!(deserialized.branch, "main");
        assert_eq!(deserialized.comments.len(), 1);
        assert_eq!(deserialized.comments[0].text, "Test comment");
    }

    #[test]
    fn test_comments_filtering() {
        let mut commit_comments = CommitComments::new("abc123".to_string(), "main".to_string());

        commit_comments.add_comment(Comment::new_line(
            "src/main.rs".to_string(),
            42,
            LineType::Added,
            "Line comment".to_string(),
        ));
        commit_comments.add_comment(Comment::new_file(
            "src/main.rs".to_string(),
            "File comment".to_string(),
        ));
        commit_comments.add_comment(Comment::new_line(
            "src/other.rs".to_string(),
            10,
            LineType::Removed,
            "Other file".to_string(),
        ));

        let main_comments = commit_comments.comments_for_file("src/main.rs");
        assert_eq!(main_comments.len(), 2);

        let line_comments = commit_comments.comments_at_line("src/main.rs", 42, LineType::Added);
        assert_eq!(line_comments.len(), 1);

        let file_comments = commit_comments.file_level_comments("src/main.rs");
        assert_eq!(file_comments.len(), 1);
    }
}
