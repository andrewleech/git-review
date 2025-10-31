mod app;
mod comments;
mod config;
mod export;
mod git;
mod input;
mod ui;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use git2::Repository;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ExportFormatArg {
    Markdown,
    Json,
}

#[derive(Parser, Debug)]
#[command(name = "git-review")]
#[command(version, about = "Terminal-based code review tool for git branches", long_about = None)]
struct Args {
    /// Path to git repository (defaults to current directory)
    #[arg(short, long, value_name = "PATH")]
    path: Option<PathBuf>,

    /// Base branch to compare against
    #[arg(short, long, value_name = "BRANCH", conflicts_with = "range")]
    base: Option<String>,

    /// Git commit range to review
    ///
    /// Examples:
    ///   HEAD~5..HEAD    Review last 5 commits
    ///   origin/main     Compare current branch to origin/main
    ///   v1.0..v2.0      Review commits between tags
    #[arg(short, long, value_name = "RANGE", conflicts_with = "base")]
    range: Option<String>,

    /// Initial context lines for diffs
    #[arg(short, long, value_name = "LINES", default_value = "8")]
    context: u32,

    /// Export all comments for current branch to stdout
    #[arg(long, conflicts_with = "clear_comments")]
    export_comments: bool,

    /// Export format (markdown or json)
    #[arg(
        long,
        value_enum,
        default_value = "markdown",
        requires = "export_comments"
    )]
    format: ExportFormatArg,

    /// Clear all comments for current branch
    #[arg(long, conflicts_with = "export_comments")]
    clear_comments: bool,

    /// Skip confirmation prompts (for automation)
    #[arg(short = 'y', long)]
    yes: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Open git repository
    let repo_path = args.path.unwrap_or_else(|| PathBuf::from("."));
    let repo = Repository::discover(&repo_path)
        .context("Failed to find git repository. Make sure you're in a git directory.")?;

    // Handle export comments command
    if args.export_comments {
        let branch = get_current_branch(&repo)?;
        let comments = git::notes::read_all_for_branch(&repo, &branch)?;

        if comments.is_empty() {
            eprintln!("No comments found for branch '{branch}'");
            return Ok(());
        }

        let output = match args.format {
            ExportFormatArg::Markdown => export::to_markdown(&comments, &branch)?,
            ExportFormatArg::Json => export::to_json(&comments)?,
        };

        println!("{output}");
        return Ok(());
    }

    // Handle clear comments command
    if args.clear_comments {
        let branch = get_current_branch(&repo)?;

        let confirmed = if args.yes {
            true
        } else {
            print!("Delete all comments for branch '{branch}'? [y/N]: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            input.trim().to_lowercase() == "y"
        };

        if confirmed {
            let deleted = git::notes::clear_branch_notes(&repo, &branch)?;
            println!("✓ Cleared {deleted} comment(s) for branch '{branch}'");
        } else {
            println!("Cancelled");
        }
        return Ok(());
    }

    // Get commits and base branch based on --range or --base
    let using_range = args.range.is_some();
    let (commits, base_branch) = if let Some(range) = args.range {
        // Use explicit range
        let (start_ref, end_ref) = git::parse_range(&range)?;
        let commits = git::get_commit_log_range(&repo, &start_ref, &end_ref)?;
        (commits, start_ref)
    } else {
        // Use base branch (auto-detect or explicit)
        let base = if let Some(base) = args.base {
            base
        } else {
            git::detect_base_branch(&repo)?
        };
        let commits = git::get_commit_log(&repo, &base)?;
        (commits, base)
    };

    // Load configuration
    let mut config = config::Config::load_or_default()?;
    config.display.context_lines = args.context;

    if commits.is_empty() {
        if using_range {
            println!("No commits found in specified range");
            println!("The refs point to the same commit or have no differences.");
        } else {
            println!("No commits found between HEAD and {base_branch}");
            println!("Your branch is up to date with the base branch.");
        }
        return Ok(());
    }

    // Get current branch for comment storage
    let current_branch = get_current_branch(&repo)?;

    // Initialize app state
    let mut app = app::App::new(repo, commits, config, current_branch);

    // Load comments from git notes
    app.load_comments();

    // Load initial diff
    app.init_diff();

    // Run TUI
    run_tui(app)?;

    Ok(())
}

fn run_tui(mut app: app::App) -> Result<()> {
    // Set up terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;

    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    // Run app loop
    let result = app_loop(&mut terminal, &mut app);

    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn app_loop(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    app: &mut app::App,
) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|f| {
            if let Err(e) = ui::render(f, app) {
                eprintln!("Render error: {e}");
            }
        })?;

        // Handle input
        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            match crossterm::event::read()? {
                crossterm::event::Event::Key(key) => {
                    if input::handle_key_event(key, app)? {
                        break; // Exit requested
                    }
                }
                crossterm::event::Event::Mouse(mouse) => {
                    input::handle_mouse_event(mouse, app)?;
                }
                crossterm::event::Event::Resize(width, height) => {
                    app.handle_resize(width, height);
                }
                _ => {}
            }
        }
    }

    Ok(())
}

/// Get the current branch name
fn get_current_branch(repo: &Repository) -> Result<String> {
    let head = repo.head().context("Failed to get HEAD reference")?;

    if let Some(branch_name) = head.shorthand() {
        // Remove "refs/heads/" prefix if present
        let name = branch_name
            .strip_prefix("refs/heads/")
            .unwrap_or(branch_name);
        Ok(name.to_string())
    } else {
        // Detached HEAD state - use commit SHA
        let oid = head.target().context("HEAD has no target")?;
        Ok(format!("detached-{}", &oid.to_string()[..7]))
    }
}
