pub mod app;
pub mod browser;
pub mod editor;
pub mod effects;
pub mod frontmatter;
pub mod theme;
pub mod ui;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;

pub fn run_editor(file: Option<PathBuf>) -> Result<()> {
    install_panic_hook();

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, crossterm::cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(file)?;
    let result = event_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::cursor::Show,
    )?;
    terminal.show_cursor()?;

    result
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    let target_frame = Duration::from_millis(16);
    let mut last_frame = Instant::now();

    loop {
        let elapsed = last_frame.elapsed();
        last_frame = Instant::now();

        terminal.draw(|f| {
            ui::draw(f, app);
            if app.effects.is_running() {
                app.effects.process_frame(elapsed, f.buffer_mut(), app.last_full_area);
            }
        })?;

        let remaining = target_frame.saturating_sub(last_frame.elapsed());
        if event::poll(remaining)? {
            let ev = event::read()?;
            app.handle_event(&ev)?;
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(
            std::io::stdout(),
            LeaveAlternateScreen,
            crossterm::cursor::Show,
        );
        original(info);
    }));
}
