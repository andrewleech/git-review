use crate::comments::{Comment, CommentLevel, CommitComments};
use crate::config::Config;
use crate::git::{CommitInfo, FileDiff, LineType};
use git2::Repository;
use std::collections::HashMap;

// Implementation submodules
mod comments;
mod diff;
mod navigation;
mod search;
mod view;

/// Comment mode state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommentMode {
    /// Normal viewing mode
    Normal,
    /// Creating a new comment
    Creating {
        level: CommentLevel,
        file_path: String,
        line_number: Option<usize>,
        line_type: Option<LineType>,
        hunk_header: Option<String>,
    },
    /// Viewing comments at current location
    ViewingComments(Vec<Comment>),
}

/// Search mode state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchMode {
    /// Not searching
    Normal,
    /// User is typing search query
    Entering,
}

/// A single search match location
#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub line_index: usize,  // Global line index in diff
    pub char_start: usize,  // Character offset in line
    pub char_end: usize,    // End of match
}

/// Application state
pub struct App {
    pub repo: Repository,
    pub commits: Vec<CommitInfo>,
    pub config: Config,

    // UI state
    pub selected_commit_index: usize,
    pub selected_file_index: usize,
    pub log_pane_visible: bool,
    pub help_visible: bool,
    pub scroll_offset: usize,
    pub horizontal_scroll: usize, // Horizontal scroll offset for side-by-side mode
    pub cursor_line: usize,       // Current line in diff view
    pub terminal_width: u16,
    pub terminal_height: u16,

    // Current diff data
    pub current_files: Vec<FileDiff>,
    pub current_context_lines: u32, // Context lines for current diff

    // Comment system state
    pub comment_mode: CommentMode,
    pub comment_draft: String,
    pub comments_by_commit: HashMap<String, CommitComments>, // commit_id -> comments
    pub current_branch: String,
    pub status_message: Option<String>, // For error/success messages

    // Search state
    pub search_mode: SearchMode,
    pub search_query: String,
    pub search_matches: Vec<SearchMatch>,
    pub current_match_index: Option<usize>,
}

impl App {
    pub fn new(
        repo: Repository,
        commits: Vec<CommitInfo>,
        config: Config,
        current_branch: String,
    ) -> Self {
        // Get initial terminal size
        let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));

        let initial_context = config.display.context_lines;

        Self {
            repo,
            commits,
            config,
            selected_commit_index: 0,
            selected_file_index: 0,
            log_pane_visible: true,
            help_visible: false,
            scroll_offset: 0,
            horizontal_scroll: 0,
            cursor_line: 0,
            terminal_width: width,
            terminal_height: height,
            current_files: Vec::new(),
            current_context_lines: initial_context,
            comment_mode: CommentMode::Normal,
            comment_draft: String::new(),
            comments_by_commit: HashMap::new(),
            current_branch,
            status_message: None,
            search_mode: SearchMode::Normal,
            search_query: String::new(),
            search_matches: Vec::new(),
            current_match_index: None,
        }
    }
}
