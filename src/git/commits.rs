use anyhow::{Context, Result};
use git2::{Oid, Repository, Time};

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub id: Oid,
    pub short_id: String,
    pub message: String,
    pub author_name: String,
    pub time: Time,
}

impl CommitInfo {
    /// Get the first line of the commit message
    pub fn summary(&self) -> &str {
        self.message.lines().next().unwrap_or("")
    }

    /// Format time as relative (e.g., "2 hours ago")
    pub fn relative_time(&self) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let commit_time = self.time.seconds();
        let diff = now - commit_time;

        if diff < 60 {
            "just now".to_string()
        } else if diff < 3600 {
            format!("{} minutes ago", diff / 60)
        } else if diff < 86400 {
            format!("{} hours ago", diff / 3600)
        } else if diff < 2592000 {
            format!("{} days ago", diff / 86400)
        } else if diff < 31536000 {
            format!("{} months ago", diff / 2592000)
        } else {
            format!("{} years ago", diff / 31536000)
        }
    }
}

/// Parse a git range string into start and end refs
///
/// Supports two formats:
/// - "ref" -> (HEAD, ref)
/// - "start..end" -> (start, end)
pub fn parse_range(range: &str) -> Result<(String, String)> {
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
        Ok(("HEAD".to_string(), range.to_string()))
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
        .context(format!("Failed to find ref: {}", start_ref))?;
    let start_oid = start_obj.id();

    let end_obj = repo
        .revparse_single(end_ref)
        .context(format!("Failed to find ref: {}", end_ref))?;
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
            short_id: format!("{:.7}", oid),
            message: commit
                .message()
                .unwrap_or("<no message>")
                .trim()
                .to_string(),
            author_name: commit.author().name().unwrap_or("<unknown>").to_string(),
            time: commit.time(),
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
            author_name: "Test Author".to_string(),
            time: Time::new(0, 0),
        };

        assert_eq!(commit.summary(), "First line");
    }

    #[test]
    fn test_commit_info_summary_single_line() {
        let commit = CommitInfo {
            id: Oid::zero(),
            short_id: "abc123".to_string(),
            message: "Single line message".to_string(),
            author_name: "Test Author".to_string(),
            time: Time::new(0, 0),
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
        assert_eq!(result.0, "HEAD");
        assert_eq!(result.1, "origin/main");
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
