use crate::editor::{self, Backend, Frontend};
use crate::utf8_utils::{UTF8IntoIter, Utf8ToBytes};
use std::io::Stdout;
use std::io::Write;

use crossterm::{
    event::{read, Event, KeyCode},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    QueueableCommand,
};


pub struct Terminal {
    stdout: Stdout,
    frontend_buffer: Vec<u8>,
    frontend_size: (u16, u16),
}

impl Frontend for Terminal {
    fn new() -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            stdout: std::io::stdout(),
            frontend_buffer: Vec::new(),
            frontend_size: (0, 0),
        })
    }

    fn setup(&mut self) -> anyhow::Result<()> {
        enable_raw_mode()?;
        self.stdout.queue(EnterAlternateScreen)?;
        self.stdout.queue(Clear(ClearType::All))?;
        self.stdout.flush()?;
        Ok(())
    }

    fn draw(&mut self, backend: &Backend) -> anyhow::Result<()> {
        let cursor_pos = backend.cursor_pos;
        self.frontend_buffer.clear();

        self.stdout.queue(Clear(ClearType::All))?;

        self.stdout.queue(crossterm::cursor::MoveTo(0, 0))?;
        self.stdout.flush()?;

        for glytph in backend.file_buffer.utf8_iter() {
            match glytph {
                b"\n" => {
                    self.frontend_buffer.extend_from_slice(b"\n\r");
                }
                glytph => {
                    self.frontend_buffer.extend_from_slice(glytph);
                }
            }
        }

        self.stdout.write_all(&self.frontend_buffer)?;

        self.stdout
            .queue(crossterm::cursor::MoveTo(cursor_pos as u16, 0))?;
        self.stdout.flush()?;

        Ok(())
    }

    fn event(&mut self) -> anyhow::Result<editor::InputEvent> {
        let event = crossterm::event::read()?;
        match event {
            Event::Resize(width, height) => {
                self.frontend_size = (width, height);
            }
            _ => {}
        }
        Ok(event.into())
    }

    fn cleanup(&mut self) -> anyhow::Result<()> {
        disable_raw_mode()?;
        self.stdout.queue(LeaveAlternateScreen)?;
        self.stdout.flush()?;
        Ok(())
    }
}

impl From<crossterm::event::Event> for editor::InputEvent {
    fn from(value: crossterm::event::Event) -> Self {
        match value {
            crossterm::event::Event::Key(key) => key.into(),
            //crossterm::event::Event::Resize(x, y) => ,
            _ => editor::InputEvent::Key(editor::EventKey::Null),
        }
    }
}

impl From<crossterm::event::KeyEvent> for editor::InputEvent {
    fn from(value: crossterm::event::KeyEvent) -> Self {
        match value.code {
            crossterm::event::KeyCode::Char(c) => editor::InputEvent::Key(
                c.utf8_to_bytes()
                    .map(|(size, buffer)| editor::EventKey::Char { size, buffer })
                    .unwrap_or(editor::EventKey::Null),
            ),
            crossterm::event::KeyCode::Esc => {
                editor::InputEvent::Key(editor::EventKey::Etc)
            }
            crossterm::event::KeyCode::Enter => {
                editor::InputEvent::Key(editor::EventKey::Enter)
            }
            crossterm::event::KeyCode::Backspace => {
                editor::InputEvent::Key(editor::EventKey::Backspace)
            }
            crossterm::event::KeyCode::Left => {
                editor::InputEvent::Key(editor::EventKey::Left)
            }
            crossterm::event::KeyCode::Right => {
                editor::InputEvent::Key(editor::EventKey::Right)
            }
            crossterm::event::KeyCode::Up => editor::InputEvent::Key(editor::EventKey::Up),
            crossterm::event::KeyCode::Down => {
                editor::InputEvent::Key(editor::EventKey::Down)
            }
            _ => editor::InputEvent::Key(editor::EventKey::Null),
        }
    }
}