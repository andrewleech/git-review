use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LineType {
    Old,
    New,
    Context,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewComment {
    pub file_path: String,
    pub line_number: usize,
    pub line_type: LineType,
    pub comment: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSession {
    pub commit_hash: String,
    pub branch_name: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub review_date: DateTime<Utc>,
    pub comments: Vec<ReviewComment>,
}

impl ReviewSession {
    pub fn new(commit_hash: String, branch_name: String) -> Self {
        Self {
            commit_hash,
            branch_name,
            review_date: Utc::now(),
            comments: Vec::new(),
        }
    }

    pub fn add_comment(&mut self, comment: ReviewComment) {
        self.comments.push(comment);
    }

    /// Save session to a timestamped file
    pub fn save(&self, repo_path: &std::path::Path) -> Result<PathBuf> {
        // Canonicalize repo path to prevent path traversal
        let canonical_repo = repo_path.canonicalize()
            .context("Failed to canonicalize repository path")?;

        let review_dir = canonical_repo.join(".git-review");
        fs::create_dir_all(&review_dir)
            .context("Failed to create .git-review directory")?;

        // Generate filename with timestamp and process ID for uniqueness
        let filename = format!(
            "review-{}-{}.txt",
            self.review_date.format("%Y-%m-%d-%H%M%S"),
            std::process::id()
        );
        let file_path = review_dir.join(&filename);

        // Verify the file path is still within the review directory
        if let Ok(canonical_file) = file_path.canonicalize() {
            if !canonical_file.starts_with(&review_dir) {
                anyhow::bail!("Attempted path traversal detected");
            }
        }

        // Format as human-readable text
        let content = self.to_text();

        fs::write(&file_path, content)
            .context(format!("Failed to write review file: {}", file_path.display()))?;

        Ok(file_path)
    }

    /// Format session as human-readable text
    pub fn to_text(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("Review Date: {}\n", self.review_date.format("%Y-%m-%d %H:%M:%S")));
        output.push_str(&format!("Commit: {}\n", self.commit_hash));
        output.push_str(&format!("Branch: {}\n", self.branch_name));
        output.push_str("\n---\n\n");

        for comment in &self.comments {
            let line_type_str = match comment.line_type {
                LineType::Old => "old",
                LineType::New => "new",
                LineType::Context => "context",
            };

            output.push_str(&format!("File: {}\n", comment.file_path));
            output.push_str(&format!("Line: {} ({})\n", comment.line_number, line_type_str));
            output.push_str("Comment:\n");
            output.push_str(&comment.comment);
            output.push_str("\n\n---\n\n");
        }

        output
    }

    /// Load all review sessions from .git-review directory
    pub fn load_all(repo_path: &std::path::Path) -> Result<Vec<ReviewSession>> {
        let review_dir = repo_path.join(".git-review");

        if !review_dir.exists() {
            return Ok(Vec::new());
        }

        let sessions = Vec::new();

        for entry in fs::read_dir(&review_dir)
            .context("Failed to read .git-review directory")?
        {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("txt") {
                // For now, we just track that sessions exist
                // Full parsing could be added later
            }
        }

        Ok(sessions)
    }
}

pub struct CommentManager {
    current_session: Option<ReviewSession>,
    comments_by_location: HashMap<(String, usize), Vec<ReviewComment>>,
}

impl CommentManager {
    pub fn new() -> Self {
        Self {
            current_session: None,
            comments_by_location: HashMap::new(),
        }
    }

    pub fn start_session(&mut self, commit_hash: String, branch_name: String) {
        self.current_session = Some(ReviewSession::new(commit_hash, branch_name));
    }

    pub fn add_comment(
        &mut self,
        file_path: String,
        line_number: usize,
        line_type: LineType,
        comment: String,
    ) {
        let review_comment = ReviewComment {
            file_path: file_path.clone(),
            line_number,
            line_type,
            comment,
            timestamp: Utc::now(),
        };

        // Add to current session
        if let Some(session) = &mut self.current_session {
            session.add_comment(review_comment.clone());
        }

        // Index by location for quick lookup
        self.comments_by_location
            .entry((file_path, line_number))
            .or_default()
            .push(review_comment);
    }

    pub fn get_comments(&self, file_path: &str, line_number: usize) -> Option<&Vec<ReviewComment>> {
        self.comments_by_location
            .get(&(file_path.to_string(), line_number))
    }

    pub fn has_comment(&self, file_path: &str, line_number: usize) -> bool {
        self.comments_by_location
            .contains_key(&(file_path.to_string(), line_number))
    }

    pub fn save_session(&self, repo_path: &std::path::Path) -> Result<Option<PathBuf>> {
        if let Some(session) = &self.current_session {
            let path = session.save(repo_path)?;
            Ok(Some(path))
        } else {
            Ok(None)
        }
    }
}

impl Default for CommentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_session_creation() {
        let session = ReviewSession::new("abc123".to_string(), "feature/test".to_string());
        assert_eq!(session.commit_hash, "abc123");
        assert_eq!(session.branch_name, "feature/test");
        assert!(session.comments.is_empty());
    }

    #[test]
    fn test_add_comment() {
        let mut manager = CommentManager::new();
        manager.start_session("abc123".to_string(), "main".to_string());

        manager.add_comment(
            "src/main.rs".to_string(),
            42,
            LineType::New,
            "This needs refactoring".to_string(),
        );

        assert!(manager.has_comment("src/main.rs", 42));
        assert!(!manager.has_comment("src/main.rs", 43));
    }

    #[test]
    fn test_to_text_format() {
        let mut session = ReviewSession::new("abc123".to_string(), "main".to_string());
        session.add_comment(ReviewComment {
            file_path: "src/main.rs".to_string(),
            line_number: 42,
            line_type: LineType::New,
            comment: "Test comment".to_string(),
            timestamp: Utc::now(),
        });

        let text = session.to_text();
        assert!(text.contains("Commit: abc123"));
        assert!(text.contains("src/main.rs"));
        assert!(text.contains("Test comment"));
    }
}
