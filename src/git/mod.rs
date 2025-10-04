pub mod branch;
pub mod commits;
pub mod diff;
pub mod diff_parser;

pub use branch::detect_base_branch;
pub use commits::{get_commit_log, CommitInfo};
pub use diff::{diff_to_text, generate_diff, DiffOptions};
pub use diff_parser::{parse_diff, FileDiff, Hunk, HunkLine, LineType};
