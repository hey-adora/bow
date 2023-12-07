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

struct Editor<'a> {
    line: &'a str,
}

impl<'a> Widget for Editor<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // let mut temp = [0; 4];
        // let x = self.line.encode_utf8(&mut temp);
        // buf.set_string(0, 0, string, style)
        // buf.get_mut(0, 0).set_string(self.line);

        buf.set_string(0, 0, self.line, Style::new());
    }
}

impl<'a> Editor<'a> {
    pub fn new(c: &'a str) -> Self {
        Editor { line: c }
    }
}

fn main() {
    let mut terminal = setup_terminal().unwrap();
    run(&mut terminal).unwrap();
    restore_terminal(&mut terminal).unwrap();
}

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), RunError> {
    // terminal.flush()?;
    // let mut switch = true;
    // let mut line: &str = "";
    let mut line: String = String::new();
    loop {
        terminal.draw(|frame| {
            frame.render_widget(Editor::new(&line), frame.size());
        })?;

        // if should_quit()? {
        //     break;
        // }

        if let Event::Key(key) = event::read()? {
            if let KeyCode::Esc = key.code {
                break;
            } else if let KeyCode::Backspace = key.code {
                line.pop();
            } else if let KeyCode::Char(key) = key.code {
                line.push(key);
            }
        }

        // let b = terminal.current_buffer_mut();
        // let p = b.get_mut(0, 0);
        // p.set_symbol(match switch {
        //     true => {
        //         switch = false;
        //         "x"
        //     }
        //     false => {
        //         switch = true;
        //         "y"
        //     }
        // });
        // terminal.flush()?;
        // terminal.hide_cursor()?;
        // terminal.swap_buffers();
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
