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

/// Get commit log between HEAD and base branch
///
/// Returns commits in reverse chronological order (newest first)
pub fn get_commit_log(repo: &Repository, base_branch: &str) -> Result<Vec<CommitInfo>> {
    let head = repo.head().context("Failed to get HEAD reference")?;
    let head_oid = head
        .target()
        .context("HEAD does not point to a valid commit")?;

    let base_ref = repo
        .revparse_single(base_branch)
        .context(format!("Failed to find base branch: {}", base_branch))?;
    let base_oid = base_ref.id();

    // If HEAD and base are the same, return empty vec
    if head_oid == base_oid {
        return Ok(Vec::new());
    }

    let mut revwalk = repo.revwalk()?;
    revwalk.push(head_oid)?;
    revwalk.hide(base_oid)?;
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
}
