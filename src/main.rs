mod app;
mod config;
mod git;
mod input;
mod ui;

use anyhow::{Context, Result};
use clap::Parser;
use git2::Repository;
use std::path::PathBuf;

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
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Open git repository
    let repo_path = args.path.unwrap_or_else(|| PathBuf::from("."));
    let repo = Repository::discover(&repo_path)
        .context("Failed to find git repository. Make sure you're in a git directory.")?;

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

    // Initialize app state
    let mut app = app::App::new(repo, commits, config);

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
