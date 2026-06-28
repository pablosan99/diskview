use std::{
    io,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc, Arc,
    },
    thread,
    time::{Duration, SystemTime},
};

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::ListState};

mod app;
mod cache;
mod scanner;
mod ui;

use app::{App, ConfirmAction};

#[derive(Parser)]
#[command(name = "diskview", about = "Interactive terminal disk usage analyzer")]
struct Cli {
    /// Directory to scan (defaults to current directory)
    path: Option<PathBuf>,

    /// Force rescan, ignoring cache
    #[arg(short, long)]
    refresh: bool,

    /// Cache expiry in hours (default: 24)
    #[arg(short, long, default_value = "24")]
    expiry: u64,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let scan_path = cli
        .path
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let scan_path = scan_path.canonicalize().unwrap_or(scan_path);

    let cached = if cli.refresh {
        cache::invalidate(&scan_path);
        None
    } else {
        cache::load(&scan_path, cli.expiry)
    };

    let (root, scanned_at) = if let Some(hit) = cached {
        hit
    } else {
        let root = scan_with_progress(&scan_path, "Scanning")?;
        let scanned_at = SystemTime::now();
        let _ = cache::save(&scan_path, &root);
        (root, scanned_at)
    };

    run_tui(root, scanned_at, scan_path)?;

    Ok(())
}

fn delete_path(path: &std::path::Path) -> Result<()> {
    if path.is_dir() {
        std::fs::remove_dir_all(path)?;
    } else {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

fn clean_path(path: &std::path::Path) -> Result<()> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            std::fs::remove_dir_all(&p)?;
        } else {
            std::fs::remove_file(&p)?;
        }
    }
    Ok(())
}

fn scan_with_progress(path: &std::path::Path, label: &str) -> Result<scanner::DirNode> {
    let progress = Arc::new(AtomicU64::new(0));
    let progress_clone = Arc::clone(&progress);
    let path_buf = path.to_path_buf();
    let label = label.to_string();

    let (tx, rx) = mpsc::channel::<Result<scanner::DirNode>>();

    thread::spawn(move || {
        let _ = tx.send(scanner::scan(&path_buf, progress_clone));
    });

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = loop {
        let count = progress.load(Ordering::Relaxed);
        terminal.draw(|f| ui::draw_scanning(f, count, &label))?;

        match rx.try_recv() {
            Ok(result) => break result,
            Err(mpsc::TryRecvError::Empty) => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                break Err(anyhow::anyhow!("Scanner thread panicked"));
            }
        }
    };

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    result
}

fn run_tui(
    mut root: scanner::DirNode,
    mut scanned_at: SystemTime,
    root_path: PathBuf,
) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = tui_loop(&mut terminal, &mut root, &mut scanned_at, &root_path);

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    result
}

fn tui_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    root: &mut scanner::DirNode,
    scanned_at: &mut SystemTime,
    root_path: &std::path::Path,
) -> Result<()> {
    let mut app = App::new(root.clone(), *scanned_at, root_path.to_path_buf());
    let mut list_state = ListState::default();
    list_state.select(Some(0));

    loop {
        list_state.select(Some(app.selected));
        terminal.draw(|f| ui::draw(f, &app, &mut list_state))?;

        if !event::poll(Duration::from_millis(250))? {
            continue;
        }

        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        if app.confirm.is_some() {
            match key.code {
                KeyCode::Char('y') | KeyCode::Enter => {
                    if let Some((action, path)) = app.confirm.take() {
                        let op_result = match action {
                            ConfirmAction::Delete => delete_path(&path),
                            ConfirmAction::Clean => clean_path(&path),
                        };
                        if let Err(e) = op_result {
                            // Re-draw with error would require extra state; for now just ignore
                            // and fall through to rescan so the UI stays consistent.
                            let _ = e;
                        }
                        disable_raw_mode()?;
                        io::stdout().execute(LeaveAlternateScreen)?;

                        cache::invalidate(root_path);
                        let new_root = scan_with_progress(root_path, "Rescanning")?;
                        let new_scanned_at = SystemTime::now();
                        let _ = cache::save(root_path, &new_root);

                        enable_raw_mode()?;
                        io::stdout().execute(EnterAlternateScreen)?;
                        terminal.clear()?;

                        app = App::new(new_root, new_scanned_at, root_path.to_path_buf());
                        list_state.select(Some(0));
                    }
                }
                KeyCode::Char('n') | KeyCode::Esc => app.cancel_confirm(),
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => app.enter(),
                KeyCode::Backspace | KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('u') => {
                    app.go_up()
                }
                KeyCode::Char('s') => app.cycle_sort(),
                KeyCode::Char('d') => app.start_confirm(ConfirmAction::Delete),
                KeyCode::Char('c') => app.start_confirm(ConfirmAction::Clean),
                KeyCode::Char('r') => {
                    disable_raw_mode()?;
                    io::stdout().execute(LeaveAlternateScreen)?;

                    cache::invalidate(root_path);
                    let new_root = scan_with_progress(root_path, "Rescanning")?;
                    let new_scanned_at = SystemTime::now();
                    let _ = cache::save(root_path, &new_root);

                    enable_raw_mode()?;
                    io::stdout().execute(EnterAlternateScreen)?;
                    terminal.clear()?;

                    app = App::new(new_root, new_scanned_at, root_path.to_path_buf());
                    list_state.select(Some(0));
                }
                _ => {}
            }
        }
    }

    Ok(())
}
