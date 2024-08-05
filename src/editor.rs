pub mod backend;
pub mod frontend;

pub trait App {
    //fn new() -> A;
    fn run(&mut self) -> anyhow::Result<()>;
}

pub struct Editor<FrontendType: Frontend> {
    pub backend: Backend,
    pub frontend: FrontendType,
}

pub trait Frontend {
    fn new() -> anyhow::Result<Self>
    where
        Self: Sized;
    fn setup(&mut self) -> anyhow::Result<()>;
    fn draw(&mut self, frontend: &Backend) -> anyhow::Result<()>;
    fn event(&mut self) -> anyhow::Result<InputEvent>;
    fn cleanup(&mut self) -> anyhow::Result<()>;
}

pub struct Backend {
    pub file_buffer: Vec<u8>,
    pub show_cursor: bool,
    pub cursor_pos: usize,
    pub center_pos: usize,
}

pub enum LoopEvent {
    Exit,
    Continue,
}

pub enum InputEvent {
    Key(EventKey),
    //Resize((u16, u16)),
}

pub enum EventKey {
    Enter,
    Etc,
    Backspace,
    Left,
    Right,
    Up,
    Down,
    Char { size: u8, buffer: [u8; 4] },
    Null,
}

impl<FrontendType: Frontend> App for Editor<FrontendType> {
    fn run(&mut self) -> anyhow::Result<()> {
        self.frontend.setup()?;
        loop {
            self.frontend.draw(&self.backend)?;
            let event = self.frontend.event()?;
            match event {
                InputEvent::Key(key) => match key {
                    EventKey::Etc => {
                        break;
                    }
                    EventKey::Enter => {
                        self.backend.buffer_insert(b"\n\r");
                    }
                    EventKey::Backspace => {
                        self.backend.file_buffer.pop();
                        self.backend.cursor_pos = self.backend.cursor_pos.saturating_sub(1);
                    }
                    EventKey::Char { size, buffer } => {
                        let Some(buffer) = buffer.get(0..size as usize) else {
                            continue;
                        };
                        self.backend.buffer_insert(buffer);
                        self.backend.cursor_pos = self.backend.cursor_pos.saturating_add(1);
                    }
                    EventKey::Left => {
                        self.backend.cursor_pos = self.backend.cursor_pos.saturating_sub(1);
                        
                    }
                    EventKey::Right => {
                        if self.backend.cursor_pos < self.backend.file_buffer.len() {
                            self.backend.cursor_pos = self.backend.cursor_pos.saturating_add(1);
                        }
                        
                    }
                    EventKey::Up => {
                        self.backend.cursor_pos = self.backend.cursor_pos.saturating_sub(1);
                    }
                    EventKey::Down => {
                        self.backend.cursor_pos = self.backend.cursor_pos.saturating_add(1);
                    }
                    EventKey::Null => {}
                },
                // InputEvent::Resize((x, y)) => {
                //     self.frontend.
                // }
            };
        }
        self.frontend.cleanup()?;
        Ok(())
    }
}


impl Backend {
    pub fn new() -> Self {
        Self {
            file_buffer: Vec::new(),
            show_cursor: false,
            cursor_pos: 0,
            center_pos: 0,
        }
    }

    fn buffer_insert(&mut self, buffer: &[u8]) {
        if self.cursor_pos < self.file_buffer.len() {
            self.file_buffer.splice(
                self.cursor_pos..self.cursor_pos,
                buffer.iter().cloned(),
            );
        } else {
            self.file_buffer.extend_from_slice(buffer);
        }
    }

}

impl<FrontendType: Frontend> Editor<FrontendType> {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            frontend: FrontendType::new()?,
            backend: Backend::new(),
        })
    }

}