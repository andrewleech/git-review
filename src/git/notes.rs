use anyhow::{Context, Result};
use git2::{Oid, Repository, Signature};

use crate::comments::CommitComments;

/// Generate the git notes ref name for a specific branch
///
/// Branch names are sanitized to be valid git ref names
fn branch_ref_name(branch: &str) -> String {
    // Sanitize branch name for use in git ref
    let sanitized = branch
        .replace('/', "-")
        .replace(' ', "-")
        .replace('\\', "-");
    format!("refs/notes/git-review/{}", sanitized)
}

/// Write comments for a commit to git notes
///
/// Stores the comments as JSON in a git note attached to the commit
pub fn write_comments(
    repo: &Repository,
    branch: &str,
    commit_oid: Oid,
    comments: &CommitComments,
) -> Result<()> {
    let ref_name = branch_ref_name(branch);
    let json = comments.to_json()?;

    // Get signature for the note
    let sig = repo
        .signature()
        .or_else(|_| Signature::now("git-review", "git-review@local"))
        .context("Failed to create git signature")?;

    // Write the note (force=true to overwrite existing)
    repo.note(&sig, &sig, Some(&ref_name), commit_oid, &json, true)
        .context("Failed to write git note")?;

    Ok(())
}

/// Read comments for a single commit from git notes
///
/// Returns None if no comments exist for this commit
pub fn read_comments(repo: &Repository, branch: &str, commit_oid: Oid) -> Result<Option<CommitComments>> {
    let ref_name = branch_ref_name(branch);

    match repo.find_note(Some(&ref_name), commit_oid) {
        Ok(note) => {
            let message = note
                .message()
                .context("Failed to read note message")?;
            let comments = CommitComments::from_json(message)
                .context("Failed to parse comments JSON")?;
            Ok(Some(comments))
        }
        Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
        Err(e) => Err(e).context("Failed to read git note"),
    }
}

/// Read all comments for a branch
///
/// Returns a vector of all commit comments stored in the branch's notes
pub fn read_all_for_branch(repo: &Repository, branch: &str) -> Result<Vec<CommitComments>> {
    let ref_name = branch_ref_name(branch);
    let mut all_comments = Vec::new();

    // Check if the notes ref exists
    match repo.find_reference(&ref_name) {
        Ok(_) => {
            // Ref exists, iterate through notes
            let notes = repo
                .notes(Some(&ref_name))
                .context("Failed to list git notes")?;

            for note_id in notes {
                let (commit_oid, _note_oid) = note_id.context("Failed to read note ID")?;

                if let Some(comments) = read_comments(repo, branch, commit_oid)? {
                    all_comments.push(comments);
                }
            }
        }
        Err(e) if e.code() == git2::ErrorCode::NotFound => {
            // No notes exist for this branch, return empty vector
            return Ok(Vec::new());
        }
        Err(e) => {
            return Err(e).context("Failed to find notes reference");
        }
    }

    Ok(all_comments)
}

/// Delete a specific comment from a commit's notes
///
/// Removes the comment at the given index and rewrites the note.
/// If this was the last comment, the note is deleted entirely.
pub fn delete_comment(
    repo: &Repository,
    branch: &str,
    commit_oid: Oid,
    comment_index: usize,
) -> Result<bool> {
    let mut comments = match read_comments(repo, branch, commit_oid)? {
        Some(c) => c,
        None => return Ok(false), // No comments to delete
    };

    if comments.remove_comment(comment_index).is_none() {
        return Ok(false); // Index out of bounds
    }

    if comments.is_empty() {
        // Remove the note entirely if no comments left
        delete_commit_note(repo, branch, commit_oid)?;
    } else {
        // Rewrite the note with the updated comments
        write_comments(repo, branch, commit_oid, &comments)?;
    }

    Ok(true)
}

/// Delete all notes (comments) for a specific commit
fn delete_commit_note(repo: &Repository, branch: &str, commit_oid: Oid) -> Result<()> {
    let ref_name = branch_ref_name(branch);

    // Get signature for the operation
    let sig = repo
        .signature()
        .or_else(|_| Signature::now("git-review", "git-review@local"))
        .context("Failed to create git signature")?;

    repo.note_delete(commit_oid, Some(&ref_name), &sig, &sig)
        .context("Failed to delete git note")?;

    Ok(())
}

/// Clear all notes for a branch
///
/// Deletes all comment notes stored under the branch's notes ref
pub fn clear_branch_notes(repo: &Repository, branch: &str) -> Result<usize> {
    let ref_name = branch_ref_name(branch);
    let mut deleted_count = 0;

    // Check if the notes ref exists
    match repo.find_reference(&ref_name) {
        Ok(_) => {
            // Collect commit OIDs first to avoid borrowing issues
            let mut commit_oids = Vec::new();

            let notes = repo
                .notes(Some(&ref_name))
                .context("Failed to list git notes")?;

            for note_id in notes {
                let (commit_oid, _note_oid) = note_id.context("Failed to read note ID")?;
                commit_oids.push(commit_oid);
            }

            // Delete each note
            for commit_oid in commit_oids {
                if delete_commit_note(repo, branch, commit_oid).is_ok() {
                    deleted_count += 1;
                }
            }
        }
        Err(e) if e.code() == git2::ErrorCode::NotFound => {
            // No notes exist for this branch
            return Ok(0);
        }
        Err(e) => {
            return Err(e).context("Failed to find notes reference");
        }
    }

    Ok(deleted_count)
}

/// List all commit OIDs that have comments for a branch
pub fn list_commits_with_comments(repo: &Repository, branch: &str) -> Result<Vec<Oid>> {
    let ref_name = branch_ref_name(branch);
    let mut commit_oids = Vec::new();

    match repo.find_reference(&ref_name) {
        Ok(_) => {
            let notes = repo
                .notes(Some(&ref_name))
                .context("Failed to list git notes")?;

            for note_id in notes {
                let (commit_oid, _note_oid) = note_id.context("Failed to read note ID")?;
                commit_oids.push(commit_oid);
            }
        }
        Err(e) if e.code() == git2::ErrorCode::NotFound => {
            return Ok(Vec::new());
        }
        Err(e) => {
            return Err(e).context("Failed to find notes reference");
        }
    }

    Ok(commit_oids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comments::{Comment, LineType};
    use tempfile::TempDir;

    fn create_test_repo() -> (TempDir, Repository) {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();

        // Configure git identity for the test repo
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();

        (dir, repo)
    }

    fn create_test_commit(repo: &Repository) -> Oid {
        use std::fs;
        use std::path::Path;

        let path = Path::new(repo.path()).parent().unwrap().join("test.txt");
        fs::write(&path, "test content").unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(Path::new("test.txt")).unwrap();
        index.write().unwrap();

        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = repo.signature().unwrap();

        repo.commit(Some("HEAD"), &sig, &sig, "Test commit", &tree, &[])
            .unwrap()
    }

    #[test]
    fn test_branch_ref_name() {
        assert_eq!(branch_ref_name("main"), "refs/notes/git-review/main");
        assert_eq!(
            branch_ref_name("feature/test"),
            "refs/notes/git-review/feature-test"
        );
        assert_eq!(
            branch_ref_name("feat\\test"),
            "refs/notes/git-review/feat-test"
        );
    }

    #[test]
    fn test_write_and_read_comments() {
        let (_dir, repo) = create_test_repo();
        let commit_oid = create_test_commit(&repo);

        let mut comments = CommitComments::new(commit_oid.to_string(), "main".to_string());
        comments.add_comment(Comment::new_line(
            "test.txt".to_string(),
            1,
            LineType::Added,
            "Test comment".to_string(),
        ));

        // Write comments
        write_comments(&repo, "main", commit_oid, &comments).unwrap();

        // Read them back
        let read_comments = read_comments(&repo, "main", commit_oid).unwrap();
        assert!(read_comments.is_some());

        let read_comments = read_comments.unwrap();
        assert_eq!(read_comments.comments.len(), 1);
        assert_eq!(read_comments.comments[0].text, "Test comment");
    }

    #[test]
    fn test_read_nonexistent_comments() {
        let (_dir, repo) = create_test_repo();
        let commit_oid = create_test_commit(&repo);

        let comments = read_comments(&repo, "main", commit_oid).unwrap();
        assert!(comments.is_none());
    }

    #[test]
    fn test_clear_branch_notes() {
        let (_dir, repo) = create_test_repo();
        let commit_oid = create_test_commit(&repo);

        let mut comments = CommitComments::new(commit_oid.to_string(), "main".to_string());
        comments.add_comment(Comment::new_file(
            "test.txt".to_string(),
            "Test".to_string(),
        ));

        write_comments(&repo, "main", commit_oid, &comments).unwrap();

        // Clear all notes
        let deleted = clear_branch_notes(&repo, "main").unwrap();
        assert_eq!(deleted, 1);

        // Verify they're gone
        let comments = read_comments(&repo, "main", commit_oid).unwrap();
        assert!(comments.is_none());
    }
}
