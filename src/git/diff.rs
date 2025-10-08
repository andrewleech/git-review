use anyhow::{Context, Result};
use git2::{Diff, DiffOptions as Git2DiffOptions, Oid, Repository};

#[derive(Debug, Clone)]
pub struct DiffOptions {
    pub context_lines: u32,
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self { context_lines: 8 }
    }
}

/// Generate diff between a commit and its parent (or base branch)
///
/// For single commits, generates diff vs parent.
/// For comparing against base branch, use the commit closest to base.
pub fn generate_diff<'a>(
    repo: &'a Repository,
    commit_oid: Oid,
    options: &DiffOptions,
) -> Result<Diff<'a>> {
    let commit = repo
        .find_commit(commit_oid)
        .context("Failed to find commit")?;

    let tree = commit.tree().context("Failed to get commit tree")?;

    let parent_tree = if commit.parent_count() > 0 {
        Some(
            commit
                .parent(0)
                .context("Failed to get parent commit")?
                .tree()
                .context("Failed to get parent tree")?,
        )
    } else {
        None
    };

    let mut diff_opts = Git2DiffOptions::new();
    diff_opts.context_lines(options.context_lines);
    diff_opts.ignore_whitespace(false);

    let diff = repo
        .diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut diff_opts))
        .context("Failed to generate diff")?;

    Ok(diff)
}

/// Get diff for comparing branch HEAD against base branch
pub fn generate_branch_diff<'a>(
    repo: &'a Repository,
    base_branch: &str,
    options: &DiffOptions,
) -> Result<Diff<'a>> {
    let head = repo.head().context("Failed to get HEAD")?;
    let head_tree = head.peel_to_tree().context("Failed to peel HEAD to tree")?;

    let base_ref = repo
        .revparse_single(base_branch)
        .context(format!("Failed to find base branch: {}", base_branch))?;
    let base_tree = base_ref
        .peel_to_tree()
        .context(format!("Failed to peel {} to tree", base_branch))?;

    let mut diff_opts = Git2DiffOptions::new();
    diff_opts.context_lines(options.context_lines);
    diff_opts.ignore_whitespace(false);

    let diff = repo
        .diff_tree_to_tree(Some(&base_tree), Some(&head_tree), Some(&mut diff_opts))
        .context("Failed to generate branch diff")?;

    Ok(diff)
}

/// Convert git2 Diff to text (patch format)
pub fn diff_to_text(diff: &Diff) -> Result<String> {
    let mut patch_text = String::new();

    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        let origin = line.origin();
        let content = std::str::from_utf8(line.content()).unwrap_or("<invalid utf8>");

        // Add origin prefix for added/removed lines
        match origin {
            '+' | '-' | ' ' => {
                patch_text.push(origin);
                patch_text.push_str(content);
            }
            _ => {
                patch_text.push_str(content);
            }
        }
        true
    })
    .context("Failed to print diff")?;

    Ok(patch_text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_options_default() {
        let opts = DiffOptions::default();
        assert_eq!(opts.context_lines, 8);
    }
}
