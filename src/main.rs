use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};
use std::{
    io::{self, Stdout},
    time::Duration,
};
use thiserror::Error;

fn main() {
    let mut terminal = setup_terminal().unwrap();
    run(&mut terminal).unwrap();
    restore_terminal(&mut terminal).unwrap();
}

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), RunError> {
    // terminal.flush()?;
    let mut switch = true;
    loop {
        // terminal.draw(crate::render_app)?;

        let b = terminal.current_buffer_mut();
        let p = b.get_mut(0, 0);
        p.set_symbol(match switch {
            true => {
                switch = false;
                "x"
            }
            false => {
                switch = true;
                "y"
            }
        });
        terminal.flush()?;
        terminal.hide_cursor()?;
        terminal.swap_buffers();
        if should_quit()? {
            break;
        }
    }
    Ok(())
}

fn render_app(frame: &mut Frame) {
    let greeting = Paragraph::new("Hello World! (press 'q' to quit)");
    frame.render_widget(greeting, frame.size());
}

fn should_quit() -> Result<bool, ShouldQuitError> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            return Ok(KeyCode::Char('q') == key.code);
        }
    }

    Ok(false)
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, SetupTerminalError> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    let backend = Terminal::new(CrosstermBackend::new(stdout))?;
    Ok(backend)
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), RestoreTerminalError> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

#[derive(Error, Debug)]
pub enum ShouldQuitError {
    #[error("io: {0}")]
    IO(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum SetupTerminalError {
    #[error("io: {0}")]
    IO(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum RestoreTerminalError {
    #[error("io: {0}")]
    IO(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum RunError {
    #[error("io: {0}")]
    IO(#[from] std::io::Error),

    #[error("failed to quit: {0}")]
    ShouldQuitError(#[from] ShouldQuitError),
}
