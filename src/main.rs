use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};
use std::{
    io::{self, Stdout},
    marker::PhantomData,
    ops::Deref,
    ops::DerefMut,
    rc::Rc,
    sync::{Arc, RwLock},
    time::Duration,
};
use thiserror::Error;

struct Editor<'a, T> {
    phantom: PhantomData<&'a T>,
}

impl<'a, T> StatefulWidget for Editor<'a, T> {
    type State = AppState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        buf.set_string(0, 0, &state.line, Style::new());
    }
}

impl<'a> Editor<'a, PhantomData<()>> {
    pub fn new() -> Self {
        Editor {
            phantom: PhantomData,
        }
    }
}

struct AppState {
    line: String,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            line: String::new(),
        }
    }
}

struct App {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    state: AppState,
}

// impl Deref for App {
//     type Target = Self;
//     fn deref(&self) -> &Self::Target {
//         &self
//     }
// }
//
// impl DerefMut for App {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self
//     }
// }

impl App {
    pub fn new() -> Self {
        let terminal = App::setup_terminal().unwrap();
        let state = AppState::new();
        Self { terminal, state }
    }

    pub fn run(&mut self) -> Result<(), RunError> {
        // let mut app = Arc::new(RwLock::new(self));
        // let mut app = app.write().unwrap();
        loop {
            self.terminal.draw(|frame| {
                // let mut app = app.clone().write().unwrap();
                frame.render_stateful_widget(Editor::new(), frame.size(), &mut self.state);
            })?;

            if let Event::Key(key) = event::read()? {
                if let KeyCode::Esc = key.code {
                    break;
                } else if let KeyCode::Backspace = key.code {
                    self.state.line.pop();
                } else if let KeyCode::Char(key) = key.code {
                    self.state.line.push(key);
                }
            }
        }
        App::restore_terminal(&mut self.terminal).unwrap();
        Ok(())
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
}

fn main() {
    let mut app = App::new();
    app.run().unwrap();
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
