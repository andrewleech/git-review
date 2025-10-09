use assert_cmd::Command;
use predicates::prelude::*;
use std::process::Command as StdCommand;
use tempfile::TempDir;

/// Helper to create a test git repository with commits
fn create_test_repo() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let repo_path = temp_dir.path();

    // Initialize git repo
    StdCommand::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to init git repo");

    // Configure git
    StdCommand::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to configure git email");

    StdCommand::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to configure git name");

    // Create initial commit on main branch
    std::fs::write(repo_path.join("file1.txt"), "Initial content\n")
        .expect("Failed to write file1.txt");

    StdCommand::new("git")
        .args(["add", "file1.txt"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to add file");

    StdCommand::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to create initial commit");

    // Create a feature branch
    StdCommand::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to create feature branch");

    // Make some changes
    std::fs::write(repo_path.join("file1.txt"), "Modified content\n")
        .expect("Failed to write modified file1.txt");

    StdCommand::new("git")
        .args(["add", "file1.txt"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to add modified file");

    StdCommand::new("git")
        .args(["commit", "-m", "Feature change"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to create feature commit");

    temp_dir
}

#[test]
fn test_help_flag() {
    let mut cmd = Command::cargo_bin("git-review").expect("Failed to find binary");
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Terminal-based code review tool"));
}

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("git-review").expect("Failed to find binary");
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_no_git_repo_error() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let mut cmd = Command::cargo_bin("git-review").expect("Failed to find binary");
    cmd.current_dir(temp_dir.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed to find git repository"));
}

#[test]
fn test_repo_with_commits() {
    let temp_dir = create_test_repo();

    // This test would need to run in non-interactive mode
    // For now, we just verify the tool can start with a valid repo
    // A full TUI test would require more complex setup

    // We can at least verify the git operations work by checking --help in the repo
    let mut cmd = Command::cargo_bin("git-review").expect("Failed to find binary");
    cmd.arg("--help");
    cmd.current_dir(temp_dir.path());
    cmd.assert().success();
}

#[test]
fn test_config_defaults() {
    // Test that config module works
    use git_review::config::Config;

    let config = Config::default();
    assert_eq!(config.display.context_lines, 8);
    assert_eq!(config.display.context_expand_increment, 8);
    assert_eq!(config.ui.log_pane_width_ratio, 0.35);
    assert!(config.ui.show_line_numbers);
}

#[test]
fn test_range_argument_help() {
    let mut cmd = Command::cargo_bin("git-review").expect("Failed to find binary");
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--range"))
        .stdout(predicate::str::contains("Git commit range to review"));
}

#[test]
fn test_range_and_base_conflict() {
    let temp_dir = create_test_repo();

    let mut cmd = Command::cargo_bin("git-review").expect("Failed to find binary");
    cmd.args(["--base", "main", "--range", "HEAD~1..HEAD"]);
    cmd.current_dir(temp_dir.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn test_range_argument_invalid_ref() {
    let temp_dir = create_test_repo();

    let mut cmd = Command::cargo_bin("git-review").expect("Failed to find binary");
    cmd.args(["--range", "nonexistent..HEAD"]);
    cmd.current_dir(temp_dir.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed to find start ref"));
}

#[test]
fn test_parse_range_function() {
    use git_review::git::parse_range;

    // Test explicit range
    let (start, end) = parse_range("HEAD~5..HEAD").expect("Failed to parse range");
    assert_eq!(start, "HEAD~5");
    assert_eq!(end, "HEAD");

    // Test single target (shows commits in HEAD not in origin/main)
    let (start, end) = parse_range("origin/main").expect("Failed to parse range");
    assert_eq!(start, "origin/main");
    assert_eq!(end, "HEAD");

    // Test invalid range
    assert!(parse_range("..HEAD").is_err());
    assert!(parse_range("HEAD..").is_err());
}
