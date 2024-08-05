// use compact_str::ToCompactString;
use crossterm::{ExecutableCommand, QueueableCommand};
use editor::App;

use core::str;
use std::{
    io::{self, Stdout, Write},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::{Arc, RwLock},
    time::Duration,
};
use thiserror::Error;

// struct Editor<'a, T> {
//     phantom: PhantomData<&'a T>,
// }

pub mod editor {
    use std::marker::PhantomData;

    pub trait App {
        //fn new() -> A;
        fn run(&mut self) -> anyhow::Result<()>;
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

    pub struct Backend {
        pub file_buffer: Vec<u8>,
        pub show_cursor: bool,
        pub cursor_pos: usize,
        pub center_pos: usize,
    }

    pub struct Editor<FrontendType: frontend::Frontend> {
        pub backend: Backend,
        pub frontend: FrontendType,
    }
    //pub phantom: PhantomData<BackendType>,
    // <AppType, BackendKindType: backend::Backend<BackendType>, BackendType>
    impl<FrontendType: frontend::Frontend> App for Editor<FrontendType> {
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
                            //let (x, y) = self.backend.cursor_pos;

                            let Some(buffer) = buffer.get(0..size as usize) else {
                                continue;
                            };
                            self.backend.buffer_insert(buffer);
                            // if self.backend.cursor_pos < self.backend.file_buffer.len() {
                            //     self.backend.file_buffer.splice(
                            //         self.backend.cursor_pos..self.backend.cursor_pos,
                            //         buffer[0..size as usize].iter().cloned(),
                            //     );
                            // } else {
                            //     self.backend.file_buffer.extend_from_slice(&buffer[0..size as usize]);
                            // }

                            self.backend.cursor_pos = self.backend.cursor_pos.saturating_add(1);
                            
                            //
                        }
                        EventKey::Left => {
                            //let (x, y) = &mut self.backend.cursor_pos;
                            
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

        // fn insert_glytph(&mut self, size: u8, buffer: [u8; 4]) {
        //     if self.cursor_pos < self.file_buffer.len() {
        //         self.file_buffer.splice(
        //             self.cursor_pos..self.cursor_pos,
        //             buffer[0..size as usize].iter().cloned(),
        //         );
        //     } else {
        //         self.file_buffer.extend_from_slice(&buffer[0..size as usize]);
        //     }
        // }
    }

    impl<FrontendType: frontend::Frontend> Editor<FrontendType> {
        pub fn new() -> anyhow::Result<Self> {
            Ok(Self {
                frontend: FrontendType::new()?,
                backend: Backend::new(),
            })
        }

        

        // pub fn cleanup(&mut self) -> anyhow::Result<()> {
        //     self.backend.cleanup()?;
        //     Ok(())
        // }
    }

    // impl Editor<backend::Terminal> {
    //     fn new() -> Self {
    //         Self {
    //             backend: backend::Terminal::new(),
    //             file_buffer: Vec::new(),
    //             show_cursor: false,
    //             //phantom: PhantomData,
    //         }
    //     }
    // }

    pub mod frontend {
        use crate::editor::{self, Backend};
        use crate::utf8::{UTF8IntoIter, Utf8ToBytes};
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

        pub trait Frontend {
            fn new() -> anyhow::Result<Self>
            where
                Self: Sized;
            fn setup(&mut self) -> anyhow::Result<()>;
            fn draw(&mut self, frontend: &Backend) -> anyhow::Result<()>;
            fn event(&mut self) -> anyhow::Result<editor::InputEvent>;
            fn cleanup(&mut self) -> anyhow::Result<()>;
        }

        pub struct Terminal {
            stdout: Stdout,
            frontend_buffer: Vec<u8>,
            frontend_size: (u16, u16),
            // size: (u16, u16),
        }

        //self.stdout.queue(crossterm::cursor::Hide)?;

        // impl Terminal {
        //     pub fn new() -> Self {
        //         let stdout = std::io::stdout();
        //         Self {
        //             stdout
        //         }
        //     }
        // }

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
                //self.stdout.flush()?;

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
                //self.stdout.flush()?;

                self.stdout
                    .queue(crossterm::cursor::MoveTo(cursor_pos as u16, 0))?;
                self.stdout.flush()?;

                // let horse = [
                //     r#"Art by Brent James Benton"#, "\n\r",
                //     r#"   |\_"#, "\n\r",
                //     r#"  /  .\_"#, "\n\r",
                //     r#" |   ___)"#, "\n\r",
                //     r#" |    \"#, "\n\r",
                //     r#" |  =  |"#, "\n\r",
                //     r#" /_____\"#, "\n\r",
                //     r#"[_______]"#,
                // ].concat();

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
                            .map(|(size, buffer)| super::EventKey::Char { size, buffer })
                            .unwrap_or(super::EventKey::Null),
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
    }
}

pub mod utf8 {
    pub struct UTF8Iter<'a> {
        pub i: usize,
        pub data: &'a [u8],
        //iter: std::slice::Iter<'a, u8>
    }

    pub trait Utf8ToBytes {
        fn utf8_to_bytes(self) -> Option<(u8, [u8; 4])>;
    }

    pub trait UTF8IntoIter<'a> {
        fn utf8_iter(&'a self) -> UTF8Iter<'a>;
    }

    pub trait BitFlag {
        fn has_flag(&self, flag: Self) -> bool;
        fn has_flag_with_mask(&self, mask: Self, flag: Self) -> bool;
    }

    pub trait UTF8Flag {
        fn utf8_glyth_size(&self) -> Option<u8>;
        fn utf8_is_next_glyth(&self) -> bool;
    }

    impl Utf8ToBytes for char {
        fn utf8_to_bytes(self) -> Option<(u8, [u8; 4])> {
            let mut buffer: [u8; 4] = [0; 4];
            self.encode_utf8(&mut buffer);
            let size = buffer[0].utf8_glyth_size();
            size.map(|size| (size, buffer))
        }
    }

    impl<'a> UTF8IntoIter<'a> for &[u8] {
        fn utf8_iter(&'a self) -> UTF8Iter<'a> {
            UTF8Iter { i: 0, data: self }
        }
    }

    impl<'a> UTF8IntoIter<'a> for &str {
        fn utf8_iter(&'a self) -> UTF8Iter<'a> {
            UTF8Iter {
                i: 0,
                data: self.as_bytes(),
            }
        }
    }

    impl<'a> UTF8IntoIter<'a> for Vec<u8> {
        fn utf8_iter(&'a self) -> UTF8Iter<'a> {
            UTF8Iter { i: 0, data: &self }
        }
    }

    impl<'a> Iterator for UTF8Iter<'a> {
        type Item = &'a [u8];

        fn next(&mut self) -> Option<Self::Item> {
            let glytph_size = self.data.get(self.i)?.utf8_glyth_size()?;

            let slice = &self.data[self.i..self.i + glytph_size as usize];

            self.i += glytph_size as usize;

            Some(slice)
        }
    }

    impl BitFlag for u8 {
        fn has_flag(&self, flag: Self) -> bool {
            *self & flag == flag
        }

        fn has_flag_with_mask(&self, mask: Self, flag: Self) -> bool {
            *self & mask == flag
        }
    }

    impl UTF8Flag for u8 {
        fn utf8_glyth_size(&self) -> Option<u8> {
            let size = if self.has_flag_with_mask(0b11111_000_u8, 0b11110_000_u8) {
                4
            } else if self.has_flag_with_mask(0b1111_0000_u8, 0b1110_0000_u8) {
                3
            } else if self.has_flag_with_mask(0b111_00000_u8, 0b110_00000_u8) {
                2
            } else if self.has_flag_with_mask(0b1_0000000_u8, 0b0_0000000_u8) {
                1
            } else {
                return None;
            };

            Some(size)
        }

        fn utf8_is_next_glyth(&self) -> bool {
            self.has_flag_with_mask(0b11_000000_u8, 0b10_000000_u8)
        }
    }
}

// struct App {
//     stdout: std::io::Stdout,
//     //terminal: ratatui::Terminal<ratatui::prelude::CrosstermBackend<Stdout>>,
//     state: Editor,
// }

use crossterm::{
    event::{read, Event, KeyCode},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

fn main() -> anyhow::Result<()> {
    let mut editor = editor::Editor::<editor::frontend::Terminal>::new()?;

    editor.run()?;

    Ok(())

    // let mut stdout = std::io::stdout();
    // enable_raw_mode()?;
    // stdout.queue(EnterAlternateScreen)?;
    // stdout.queue(Clear(ClearType::All))?;
    // stdout.flush()?;

    // loop {
    //     stdout.queue(crossterm::cursor::MoveTo(0, 0))?;
    //     let horse = [
    //         r#"Art by Brent James Benton"#, "\n\r",
    //         r#"   |\_"#, "\n\r",
    //         r#"  /  .\_"#, "\n\r",
    //         r#" |   ___)"#, "\n\r",
    //         r#" |    \"#, "\n\r",
    //         r#" |  =  |"#, "\n\r",
    //         r#" /_____\"#, "\n\r",
    //         r#"[_______]"#,
    //     ].concat();
    //     stdout.write_all(horse.as_bytes())?;
    //     stdout.flush()?;

    //     if let Event::Key(key) = read()? {
    //         if let KeyCode::Esc = key.code {
    //             break;
    //         }
    //     }
    // }

    // disable_raw_mode()?;
    // stdout.execute(LeaveAlternateScreen)?;

    // Ok(())

    //let mut app = App::new();
    // //let ff = "a🦀称";
    // let ff = "🦀";

    // let a = 0_i8;
    // println!("{:#010b}", a);

    // let a = -1_i8;
    // println!("{:#010b}", a);

    // let a = -2_i8;
    // println!("{:#010b}", a);

    // let a = 127_i8;
    // println!("{:#010b}", a);

    // let a = 2_i8;
    // println!("{:#010b}", a);

    // let a = 2_i8 | 1_i8;
    // println!("{:#010b}", a);

    // let a = 2_i8;
    // println!("{:#010b}", a);

    // let a = 2_i8 & -112_i8;
    // println!("{:#010b}", a);

    // let a = 2_i8 | -112_i8;
    // println!("{:#010b}", a);

    // let a = 0_i8 | 127_i8 | -128_i8;
    // println!("{:#010b} = {}", a, a);

    // let a = -32_i8;
    // println!("{:#010b} = {}", a, a);

    // let a = 2_i8 | -32_i8;
    // println!("{:#010b} = {}", a, a);

    // let a = 2_i8 & -32_i8;
    // println!("{:#010b} = {}", a, a);

    // let a = -64_i8;
    // println!("{:#010b} = {}", a, a);

    // let a = -64_i8 | -32_i8;
    // println!("{:#010b} = {}", a, a);

    // let a = -64_i8 & -32_i8;
    // println!("{:#010b} = {}", a, a);

    // let a = -30_i8 | -32_i8;
    // println!("{:#010b} = {}", a, a);

    // let a = -30_i8 & -32_i8;
    // println!("{:#010b} = {}", a, a);

    // let x = 0b11100010_u8 as i8;
    // println!("{:#010b} = {}", x, x);

    // let x = (0b11100010_u8 & 0b1110000_u8) as i8;
    // println!("{:#010b} = {}", x, x);

    // println!("{}", 0b11100110_u8.has_flag(0b01000000_u8));
    // println!("{}", 0b11100110_u8.has_flag_with_mask(0b11000000_u8, 0b01000000_u8));

    // println!("{:08b}", 0b01100110_u8 & 0b11000000_u8);
    // println!("{}", 0b01100110_u8 & 0b11000000_u8 == 0b01000000_u8);
    // let target = 0b11100110_u8;
    // let mask = 0b11100000_u8;
    // let comp = target & mask;
    // let result = comp == mask;
    // println!("{:#010b} &  {:#010b} = {:#010b}\n{:#010b} == {:#010b} = {}", target, mask, comp, comp, target, result);

    // unsafe {
    //     let a: [u8; 4] = std::mem::transmute(0_f32);
    //     println!("{:010b} {:010b} {:010b} {:010b}", a[0], a[1], a[2], a[3]);

    //     let a: [u8; 4] = std::mem::transmute(1_f32);
    //     println!("{:010b} {:010b} {:010b} {:010b}", a[0], a[1], a[2], a[3]);

    //     // let t: f16 = 0.1;
    //     let a: [u8; 4] = std::mem::transmute(0.1_f32);
    //     println!("{:010b} {:010b} {:010b} {:010b}", a[0], a[1], a[2], a[3]);

    //     let a: [u8; 4] = std::mem::transmute(0.2_f32);
    //     println!("{:010b} {:010b} {:010b} {:010b}", a[0], a[1], a[2], a[3]);

    //     let a: [u8; 4] = std::mem::transmute(1.3_f32);
    //     println!("{:010b} {:010b} {:010b} {:010b}", a[0], a[1], a[2], a[3]);

    //     let a: [u8; 4] = std::mem::transmute(2.3_f32);
    //     println!("{:010b} {:010b} {:010b} {:010b}", a[0], a[1], a[2], a[3]);

    //     let a: [u8; 4] = std::mem::transmute(3.3_f32);
    //     println!("{:010b} {:010b} {:010b} {:010b}", a[0], a[1], a[2], a[3]);

    //     let a: [u8; 4] = std::mem::transmute(4.3_f32);
    //     println!("{:010b} {:010b} {:010b} {:010b}", a[0], a[1], a[2], a[3]);

    //     let a: [u8; 4] = std::mem::transmute(420.2_f32);
    //     println!("{:010b} {:010b} {:010b} {:010b}", a[0], a[1], a[2], a[3]);

    //     let a: [u8; 4] = std::mem::transmute(0.99999999999_f32);
    //     println!("{:010b} {:010b} {:010b} {:010b}", a[0], a[1], a[2], a[3]);
    // }

    // unsafe {
    //     let a = String::from("oggogog");
    //     let b = a;
    //     let c = a;

    //     println!("{b}, {c}");
    // }

    // let ff = "a称🦀";
    // let bytes = ff.as_bytes();

    // let mut chars: Vec<u8> =  Vec::new();

    // //let copy_count: Option<u8> = None;
    // let mut copy_count: u8 = 0;
    // let mut copy_i: u8 = 0;
    // let mut copy_buffer: [u8; 4] = [0; 4];
    // for byte in bytes {
    //     if copy_count == 0 {
    //         let new_count = byte.utf8_byte_count();
    //         match new_count {
    //             1 => {
    //                 chars.push()
    //             }
    //             2..=4 => {
    //                 copy_count = new_count - 1;
    //             }
    //             _ => {
    //                 panic!("invalid utf8 start byte");
    //             }

    //         }

    //         copy_buffer[0] = *byte;

    //         continue;
    //     }

    //     if copy_count.utf8_is_next_byte() {
    //         panic!("invalid utf8 next byte");
    //     }

    // }

    // for utf8_slice in "a\n\r称🦀".utf8_iter() {
    //     println!("{utf8_slice:?}");
    // }

    // return;

    // println!("\n");

    // let ff = str::from_utf8(ff.as_bytes()).unwrap().chars();
    // for c in ff {
    //     let c = c.to_string();
    //     let ff = c.as_bytes();

    //     for f in ff {
    //         println!("{:#b}", f);
    //     }

    //     println!("\n");
    // }

    //return;

    // println!("{:#?}", str::from_utf8(&[97, 240, 159, 166, 128, 231, 167, 176]).unwrap().chars());
    // return;

    //let size = crossterm::terminal::size().unwrap();
    //println!("size: {size:?}");
    //
    //app.run().unwrap();
}

// impl App {
//     pub fn new() -> Self {
//         //let terminal = App::setup_terminal().unwrap();

//         let stdout = std::io::stdout();
//         //let state = EditorState::new();
//         Self { stdout, state }
//     }

//     pub fn run(&mut self) -> Result<(), RunError> {
//         // let mut app = Arc::new(RwLock::new(self));
//         // let mut app = app.write().unwrap();
//         self.setup_terminal()?;
//         self.draw()?;
//         loop {
//             let exit = self.read_event()?;
//             if exit {
//                 break;
//             }
//             self.draw()?;
//         }

//         self.restore_terminal()?;
//         Ok(())
//     }

//     fn read_event(&mut self) -> Result<bool, EventError> {
//         if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
//             if let crossterm::event::KeyCode::Esc = key.code {
//                 return Ok(true);
//             } else if let crossterm::event::KeyCode::Enter = key.code {
//                 self.state.file_buffer.push(b'\n');
//                 self.state.file_buffer.push(b'\r');
//                 //self.state.y += 1;
//             } else if let crossterm::event::KeyCode::Backspace = key.code {
//                 self.state.file_buffer.pop();
//             } else if let crossterm::event::KeyCode::Char(key) = key.code {
//                 match key {
//                     'i' => {
//                         if self.state.show_cursor {
//                             self.stdout.queue(crossterm::cursor::Hide)?;
//                             self.state.show_cursor = false;
//                         } else {
//                             self.stdout.queue(crossterm::cursor::Show)?;
//                             self.state.show_cursor = true;
//                         }
//                         self.stdout.flush()?;
//                     }
//                     _ => {
//                         let mut buffer: [u8; 4] = [0; 4];
//                         key.encode_utf8(&mut buffer);
//                         self.state.file_buffer.extend(buffer);
//                     }
//                 }
//             }
//         }

//         Ok(false)
//     }

//     fn draw(&mut self) -> Result<(), DrawError> {
//         self.stdout.queue(crossterm::terminal::Clear(
//             crossterm::terminal::ClearType::All,
//         ))?;
//         self.stdout.queue(crossterm::cursor::MoveTo(0, 0))?;

//         self.state.terminal_buffer.clear();
//         for glyth in self.state.file_buffer.utf8_iter() {
//             self.state.terminal_buffer.extend_from_slice(glyth);
//         }

//         self.stdout.write_all(&self.state.terminal_buffer)?;
//         self.stdout.flush()?;
//         // self.terminal.draw(|frame| {
//         //     // let mut app = app.clone().write().unwrap();
//         //     let size = frame.size();
//         //     //println!("size: {:?}", size);
//         //     frame.render_stateful_widget(Editor::new(), size, &mut self.state);
//         // })?;

//         Ok(())
//     }

//     fn setup_terminal(
//         &mut self,
//     ) -> Result<
//         (),
//         //ratatui::prelude::Terminal<ratatui::prelude::CrosstermBackend<Stdout>>,
//         SetupTerminalError,
//     > {
//         //let mut stdout = io::stdout();
//         crossterm::terminal::enable_raw_mode()?;
//         self.stdout
//             .queue(crossterm::terminal::EnterAlternateScreen)?;
//         self.stdout.queue(crossterm::terminal::Clear(
//             crossterm::terminal::ClearType::All,
//         ))?;
//         self.state.terminal_size = crossterm::terminal::size()?;

//         //self.stdout.queue(crossterm::cursor::Hide)?;
//         self.stdout.flush()?;
//         //crossterm::execute!(&self.stdout, crossterm::terminal::EnterAlternateScreen)?;
//         // let backend =
//         //     ratatui::prelude::Terminal::new(ratatui::prelude::CrosstermBackend::new(stdout))?;
//         Ok(())
//     }

//     fn restore_terminal(
//         &mut self,
//         //terminal: &mut ratatui::prelude::Terminal<ratatui::prelude::CrosstermBackend<Stdout>>,
//     ) -> Result<(), RestoreTerminalError> {
//         crossterm::terminal::disable_raw_mode()?;
//         self.stdout
//             .execute(crossterm::terminal::LeaveAlternateScreen)?;
//         // crossterm::execute!(
//         //     &self.stdout,
//         //     //terminal.backend_mut(),
//         //     crossterm::terminal::LeaveAlternateScreen
//         // )?;
//         //terminal.show_cursor()?;
//         Ok(())
//     }
// }

// impl<'a, T> ratatui::prelude::StatefulWidget for Editor<'a, T> {
//     type State = AppState;
//     fn render(
//         self,
//         area: ratatui::prelude::Rect,
//         buf: &mut ratatui::prelude::Buffer,
//         state: &mut Self::State,
//     ) {
//         for c in state.line.chars() {
//             // let s = ratatui::buffer::Cell {

//             // };
//             //buf.content.push(ratatui::buffer::Cell::new());
//         }
//         //buf.content[0] = ratatui::buffer::Cell::new("aaaaaa\na\n\na\na\na\na");
//         //buf.set_string(0, state.y, &state.line, Style::new());
//     }
// }

// impl<'a> Editor<'a, PhantomData<()>> {
//     pub fn new() -> Self {
//         Editor {
//             phantom: PhantomData,
//         }
//     }
// }

// impl EditorState {
//     pub fn new() -> Self {
//         Self {
//             file_buffer: String::from("oh wow \n fnfsdsdfmk"),
//             terminal_buffer: String::from("oh wow \n fnfsdsdfmk a🦀b ┌称号──┐"),
//             terminal_size: (0, 0),
//             y: 0,
//             show_cursor: true,
//         }
//     }
// }

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

#[derive(Error, Debug)]
pub enum RunError {
    // #[error("io: {0}")]
    // IO(#[from] std::io::Error),
    #[error("draw error: {0}")]
    DrawError(#[from] DrawError),

    #[error("capturing event error: {0}")]
    EventError(#[from] EventError),

    #[error("prepare terminal error: {0}")]
    PrepareTerminalError(#[from] SetupTerminalError),

    #[error("restore terminal error: {0}")]
    RestoreTerminalError(#[from] RestoreTerminalError),
    // #[error("failed to quit: {0}")]
    // ShouldQuitError(#[from] ShouldQuitError),
}

#[derive(Error, Debug)]
pub enum DrawError {
    #[error("io: {0}")]
    IO(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum EventError {
    #[error("io: {0}")]
    IO(#[from] std::io::Error),
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
