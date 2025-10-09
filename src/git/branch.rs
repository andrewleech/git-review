use anyhow::Result;
use git2::Repository;

/// Detect the base branch for comparison
///
/// Tries in order: origin/main, origin/master, main, master
pub fn detect_base_branch(repo: &Repository) -> Result<String> {
    let candidates = vec!["origin/main", "origin/master", "main", "master"];

    for branch_name in candidates {
        if repo.revparse_single(branch_name).is_ok() {
            return Ok(branch_name.to_string());
        }
    }

    anyhow::bail!(
        "Could not find base branch. Tried: origin/main, origin/master, main, master.\n\
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
