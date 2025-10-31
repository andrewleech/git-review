use anyhow::Result;
use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::comments::{Comment, CommentLevel, CommitComments};

/// Format for exporting comments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Used in Phase 2 (TUI integration)
pub enum ExportFormat {
    Markdown,
    Json,
}

/// Structure for JSON export
#[derive(Debug, Serialize, Deserialize)]
struct ExportData {
    branch: String,
    exported_at: String,
    commits: Vec<ExportCommit>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExportCommit {
    id: String,
    message: Option<String>,
    timestamp: String,
    files: Vec<ExportFile>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExportFile {
    path: String,
    comments: Vec<ExportComment>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExportComment {
    level: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    line_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hunk_header: Option<String>,
    text: String,
    created_at: String,
}

/// Export all comments for a branch to markdown format
pub fn to_markdown(comments_list: &[CommitComments], branch: &str) -> Result<String> {
    let mut output = String::new();

    // Header
    output.push_str(&format!("# Code Review: {branch}\n\n"));
    output.push_str(&format!(
        "Exported: {}\n\n",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    ));

    if comments_list.is_empty() {
        output.push_str("No comments found.\n");
        return Ok(output);
    }

    output.push_str(&format!(
        "Total commits with comments: {}\n\n",
        comments_list.len()
    ));
    output.push_str("---\n\n");

    // Group by commit
    for commit_comments in comments_list {
        // Commit header (truncate to 7 chars if longer, otherwise use full ID)
        let short_id = if commit_comments.commit_id.len() > 7 {
            &commit_comments.commit_id[..7]
        } else {
            &commit_comments.commit_id
        };
        output.push_str(&format!("## Commit: {short_id}\n\n"));
        output.push_str(&format!(
            "Date: {}\n\n",
            commit_comments.timestamp.format("%Y-%m-%d %H:%M:%S")
        ));

        // Group comments by file
        let mut files_map: std::collections::HashMap<String, Vec<&Comment>> =
            std::collections::HashMap::new();

        for comment in &commit_comments.comments {
            files_map
                .entry(comment.file_path.clone())
                .or_default()
                .push(comment);
        }

        // Sort files by path
        let mut files: Vec<_> = files_map.iter().collect();
        files.sort_by_key(|(path, _)| *path);

        for (file_path, file_comments) in files {
            output.push_str(&format!("### {file_path}\n\n"));

            // Separate comments by level
            let line_comments: Vec<_> = file_comments
                .iter()
                .filter(|c| c.level == CommentLevel::Line)
                .collect();
            let hunk_comments: Vec<_> = file_comments
                .iter()
                .filter(|c| c.level == CommentLevel::Hunk)
                .collect();
            let file_level_comments: Vec<_> = file_comments
                .iter()
                .filter(|c| c.level == CommentLevel::File)
                .collect();

            // Line-level comments
            for comment in line_comments {
                output.push_str(&format!("#### {}\n\n", comment.location_desc()));
                output.push_str(&format!("**Comment:** {}\n\n", comment.text));
            }

            // Hunk-level comments
            for comment in hunk_comments {
                output.push_str(&format!("#### {}\n\n", comment.location_desc()));
                output.push_str(&format!("**Comment:** {}\n\n", comment.text));
            }

            // File-level comments
            for comment in file_level_comments {
                output.push_str("#### File-level Comment\n\n");
                output.push_str(&format!("**Comment:** {}\n\n", comment.text));
            }
        }

        output.push_str("---\n\n");
    }

    Ok(output)
}

/// Export all comments for a branch to JSON format
pub fn to_json(comments_list: &[CommitComments]) -> Result<String> {
    if comments_list.is_empty() {
        // Return empty structure
        let export = ExportData {
            branch: "".to_string(),
            exported_at: Local::now().to_rfc3339(),
            commits: Vec::new(),
        };
        return Ok(serde_json::to_string_pretty(&export)?);
    }

    let branch = &comments_list[0].branch;
    let mut export_commits = Vec::new();

    for commit_comments in comments_list {
        // Group comments by file
        let mut files_map: std::collections::HashMap<String, Vec<&Comment>> =
            std::collections::HashMap::new();

        for comment in &commit_comments.comments {
            files_map
                .entry(comment.file_path.clone())
                .or_default()
                .push(comment);
        }

        let mut export_files = Vec::new();
        for (file_path, file_comments) in files_map {
            let mut export_comments = Vec::new();

            for comment in file_comments {
                let (level, line, line_type, hunk_header) = match &comment.location {
                    crate::comments::CommentLocation::Line { number, kind } => {
                        let kind_str = match kind {
                            crate::git::LineType::Added => "added",
                            crate::git::LineType::Removed => "removed",
                            crate::git::LineType::Context => "context",
                        };
                        ("line", Some(*number), Some(kind_str.to_string()), None)
                    }
                    crate::comments::CommentLocation::Hunk { header } => {
                        ("hunk", None, None, Some(header.clone()))
                    }
                    crate::comments::CommentLocation::File => ("file", None, None, None),
                };

                export_comments.push(ExportComment {
                    level: level.to_string(),
                    line,
                    line_type,
                    hunk_header,
                    text: comment.text.clone(),
                    created_at: comment.created_at.to_rfc3339(),
                });
            }

            export_files.push(ExportFile {
                path: file_path,
                comments: export_comments,
            });
        }

        // Sort files by path
        export_files.sort_by(|a, b| a.path.cmp(&b.path));

        export_commits.push(ExportCommit {
            id: commit_comments.commit_id.clone(),
            message: None, // Could be filled in if we have access to commit info
            timestamp: commit_comments.timestamp.to_rfc3339(),
            files: export_files,
        });
    }

    let export = ExportData {
        branch: branch.clone(),
        exported_at: Local::now().to_rfc3339(),
        commits: export_commits,
    };

    Ok(serde_json::to_string_pretty(&export)?)
}

/// Export a single commit's comments to markdown
#[allow(dead_code)] // Used in Phase 2 (TUI integration)
pub fn commit_to_markdown(commit_comments: &CommitComments) -> Result<String> {
    to_markdown(&[commit_comments.clone()], &commit_comments.branch)
}

/// Export a single commit's comments to JSON
#[allow(dead_code)] // Used in Phase 2 (TUI integration)
pub fn commit_to_json(commit_comments: &CommitComments) -> Result<String> {
    to_json(&[commit_comments.clone()])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comments::Comment;
    use crate::git::LineType;

    #[test]
    fn test_markdown_export_empty() {
        let result = to_markdown(&[], "main").unwrap();
        assert!(result.contains("# Code Review: main"));
        assert!(result.contains("No comments found"));
    }

    #[test]
    fn test_markdown_export_with_comments() {
        let mut commit_comments = CommitComments::new("abc123".to_string(), "main".to_string());

        commit_comments.add_comment(
            Comment::new_line(
                "src/main.rs".to_string(),
                42,
                LineType::Added,
                "This needs error handling".to_string(),
            )
            .unwrap(),
        );

        commit_comments.add_comment(
            Comment::new_file(
                "src/main.rs".to_string(),
                "Consider refactoring".to_string(),
            )
            .unwrap(),
        );

        let result = to_markdown(&[commit_comments], "main").unwrap();

        assert!(result.contains("# Code Review: main"));
        assert!(result.contains("## Commit: abc123"));
        assert!(result.contains("### src/main.rs"));
        assert!(result.contains("This needs error handling"));
        assert!(result.contains("Consider refactoring"));
    }

    #[test]
    fn test_json_export_empty() {
        let result = to_json(&[]).unwrap();
        let parsed: ExportData = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.commits.len(), 0);
    }

    #[test]
    fn test_json_export_with_comments() {
        let mut commit_comments = CommitComments::new("abc123".to_string(), "main".to_string());

        commit_comments.add_comment(
            Comment::new_line(
                "src/main.rs".to_string(),
                42,
                LineType::Added,
                "Test comment".to_string(),
            )
            .unwrap(),
        );

        let result = to_json(&[commit_comments]).unwrap();
        let parsed: ExportData = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed.branch, "main");
        assert_eq!(parsed.commits.len(), 1);
        assert_eq!(parsed.commits[0].id, "abc123");
        assert_eq!(parsed.commits[0].files.len(), 1);
        assert_eq!(parsed.commits[0].files[0].path, "src/main.rs");
        assert_eq!(parsed.commits[0].files[0].comments.len(), 1);
        assert_eq!(parsed.commits[0].files[0].comments[0].text, "Test comment");
    }
}
