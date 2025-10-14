use anyhow::Result;
use git2::Repository;

/// Detect the base branch for comparison
///
/// Strategy:
/// 1. Try to find the upstream tracking branch of main/master
/// 2. Fall back to local main/master
/// 3. Fall back to origin/main or origin/master
pub fn detect_base_branch(repo: &Repository) -> Result<String> {
    // First, try to find main or master branches and check their upstream
    for local_branch in ["main", "master"] {
        if let Ok(branch) = repo.find_branch(local_branch, git2::BranchType::Local) {
            // Check if this branch has an upstream tracking branch
            if let Ok(upstream) = branch.upstream() {
                if let Some(upstream_name) = upstream.name()? {
                    // Verify the upstream ref exists and return it
                    if repo.revparse_single(upstream_name).is_ok() {
                        return Ok(upstream_name.to_string());
                    }
                }
            }

            // If no upstream or upstream doesn't exist, use the local branch
            if repo.revparse_single(local_branch).is_ok() {
                return Ok(local_branch.to_string());
            }
        }
    }

    // Fall back to remote branches if no local main/master exists
    let remote_candidates = vec!["origin/main", "origin/master"];
    for branch_name in remote_candidates {
        if repo.revparse_single(branch_name).is_ok() {
            return Ok(branch_name.to_string());
        }
    }

    anyhow::bail!(
        "Could not find base branch. Tried: main, master (with upstream tracking), origin/main, origin/master.\n\
         Make sure you have a main or master branch."
    )
}

#[cfg(test)]
mod tests {
    
    
    

    #[test]
    fn test_detect_base_branch_finds_main() {
        // This test would need a test repository setup
        // For now, it's a placeholder for the structure
    }
}
