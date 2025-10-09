use anyhow::{Context, Result};
use git2::{Oid, Repository};

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub id: Oid,
    pub short_id: String,
    pub message: String,
}

impl CommitInfo {
    /// Get the first line of the commit message
    pub fn summary(&self) -> &str {
        self.message.lines().next().unwrap_or("")
    }
}

/// Parse a git range string into start and end refs
///
/// Supports two formats:
/// - "ref" -> (ref, HEAD) - commits in HEAD not in ref
/// - "start..end" -> (start, end) - commits in end not in start
pub fn parse_range(range: &str) -> Result<(String, String)> {
    // Check for three-dot syntax which is not supported
    if range.contains("...") {
        anyhow::bail!(
            "Three-dot range syntax (A...B) is not supported. Use two-dot syntax (A..B) to show commits in B but not in A."
        );
    }

    if range.contains("..") {
        let parts: Vec<&str> = range.split("..").collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid range format. Use <ref> or <start>..<end>");
        }
        if parts[0].is_empty() || parts[1].is_empty() {
            anyhow::bail!("Invalid range format. Both start and end refs must be specified");
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    } else {
        // Single ref: show commits in HEAD not in the specified ref
        Ok((range.to_string(), "HEAD".to_string()))
    }
}

/// Get commit log between two arbitrary refs
///
/// Returns commits in reverse chronological order (newest first)
pub fn get_commit_log_range(
    repo: &Repository,
    start_ref: &str,
    end_ref: &str,
) -> Result<Vec<CommitInfo>> {
    let start_obj = repo
        .revparse_single(start_ref)
        .with_context(|| format!("Failed to find start ref '{start_ref}' in range"))?;
    let start_oid = start_obj.id();

    let end_obj = repo
        .revparse_single(end_ref)
        .with_context(|| format!("Failed to find end ref '{end_ref}' in range"))?;
    let end_oid = end_obj.id();

    // If start and end are the same, return empty vec
    if start_oid == end_oid {
        return Ok(Vec::new());
    }

    let mut revwalk = repo.revwalk()?;
    revwalk.push(end_oid)?;
    revwalk.hide(start_oid)?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    let mut commits = Vec::new();

    for oid_result in revwalk {
        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;

        let commit_info = CommitInfo {
            id: oid,
            short_id: format!("{oid:.7}"),
            message: commit
                .message()
                .unwrap_or("<no message>")
                .trim()
                .to_string(),
        };

        commits.push(commit_info);
    }

    Ok(commits)
}

/// Get commit log between HEAD and base branch
///
/// Returns commits in reverse chronological order (newest first)
pub fn get_commit_log(repo: &Repository, base_branch: &str) -> Result<Vec<CommitInfo>> {
    get_commit_log_range(repo, base_branch, "HEAD")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_info_summary() {
        let commit = CommitInfo {
            id: Oid::zero(),
            short_id: "abc123".to_string(),
            message: "First line\nSecond line\nThird line".to_string(),
        };

        assert_eq!(commit.summary(), "First line");
    }

    #[test]
    fn test_commit_info_summary_single_line() {
        let commit = CommitInfo {
            id: Oid::zero(),
            short_id: "abc123".to_string(),
            message: "Single line message".to_string(),
        };

        assert_eq!(commit.summary(), "Single line message");
    }

    #[test]
    fn test_parse_range_explicit() {
        let result = parse_range("HEAD~5..HEAD").unwrap();
        assert_eq!(result.0, "HEAD~5");
        assert_eq!(result.1, "HEAD");
    }

    #[test]
    fn test_parse_range_single_target() {
        let result = parse_range("origin/main").unwrap();
        assert_eq!(result.0, "origin/main");
        assert_eq!(result.1, "HEAD");
    }

    #[test]
    fn test_parse_range_three_dot_syntax_error() {
        let result = parse_range("main...feature");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Three-dot range syntax"));
    }

    #[test]
    fn test_parse_range_with_tags() {
        let result = parse_range("v1.0..v2.0").unwrap();
        assert_eq!(result.0, "v1.0");
        assert_eq!(result.1, "v2.0");
    }

    #[test]
    fn test_parse_range_invalid_empty_start() {
        let result = parse_range("..HEAD");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Both start and end refs must be specified"));
    }

    #[test]
    fn test_parse_range_invalid_empty_end() {
        let result = parse_range("HEAD..");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Both start and end refs must be specified"));
    }

    #[test]
    fn test_parse_range_invalid_too_many_dots() {
        let result = parse_range("HEAD~5..HEAD~2..HEAD");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid range format"));
    }
}
