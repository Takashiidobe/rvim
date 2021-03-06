use crate::Document;
use crate::Row;
use crate::Terminal;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Color;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::env;
use std::fmt;
use std::fs::File;
use std::time::Duration;
use std::time::Instant;

const STATUS_FG_COLOR: Color = Color::Rgb {
    r: 63,
    g: 63,
    b: 63,
};
const STATUS_BG_COLOR: Color = Color::Rgb {
    r: 239,
    g: 239,
    b: 239,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(PartialEq, Copy, Clone)]
pub enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Default, Clone, PartialEq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

struct StatusMessage {
    text: String,
    time: Instant,
}

impl StatusMessage {
    fn from<S: Into<String>>(message: S) -> Self {
        Self {
            time: Instant::now(),
            text: message.into(),
        }
    }
}

pub enum Mode {
    Normal,
    Insert,
    Visual,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Normal => write!(f, "normal mode"),
            Self::Insert => write!(f, "insert mode"),
            Self::Visual => write!(f, "visual mode"),
        }
    }
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
    status_message: StatusMessage,
    highlighted_word: Option<String>,
    mode: Mode,
    previous_characters: Vec<char>,
}

impl Editor {
    pub fn run(&mut self) {
        enable_raw_mode().unwrap();
        loop {
            if let Err(error) = self.refresh_screen() {
                die(error);
            }
            if self.should_quit {
                break;
            }
            if let Err(error) = self.process_keypress() {
                die(error);
            }
        }
        disable_raw_mode().unwrap();
    }
    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let initial_status = String::from("HELP: `/` = find | `:w` = save | `:q` = quit");

        let document = if let Some(file_name) = args.get(1) {
            let doc = Document::open(file_name);
            if let Ok(doc) = doc {
                doc
            } else {
                let _ = File::create(file_name);
                Document::open(file_name).unwrap()
            }
        } else {
            Document::default()
        };

        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            document,
            cursor_position: Position::default(),
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),
            highlighted_word: None,
            mode: Mode::Normal,
            previous_characters: vec![],
        }
    }

    fn refresh_screen(&mut self) -> Result<(), std::io::Error> {
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position::default());
        if self.should_quit {
            Terminal::clear_screen();
        } else {
            self.document.highlight(
                &self.highlighted_word,
                Some(
                    self.offset
                        .y
                        .saturating_add(self.terminal.size().height as usize),
                ),
            );
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            if self.cursor_position.y == 0 {
                Terminal::cursor_position(&Position {
                    x: self.cursor_position.x.saturating_add(5),
                    y: self.cursor_position.y.saturating_sub(self.offset.y),
                });
            } else {
                Terminal::cursor_position(&Position {
                    x: self.cursor_position.x.saturating_sub(self.offset.x),
                    y: self.cursor_position.y.saturating_sub(self.offset.y),
                });
            }
        }
        Terminal::cursor_show();
        Terminal::flush()
    }
    fn save(&mut self) {
        if self.document.file_name.is_none() {
            let new_name = self.prompt("Save as: ", |_, _, _| {}).unwrap_or(None);
            if new_name.is_none() {
                self.status_message = StatusMessage::from("Save aborted.".to_string());
                return;
            }
            self.document.file_name = new_name;
        }

        if self.document.save().is_ok() {
            self.status_message = StatusMessage::from("File saved successfully.".to_string());
        } else {
            self.status_message = StatusMessage::from("Error writing file!".to_string());
        }
    }
    fn search(&mut self) {
        let old_position = self.cursor_position.clone();
        let mut direction = SearchDirection::Forward;
        let query = self
            .prompt(
                "Search (ESC to cancel, Arrows to navigate): ",
                |editor, key, query| {
                    let mut moved = false;
                    match key {
                        Event::Key(KeyEvent {
                            code: KeyCode::Char('n'),
                            ..
                        })
                        | Event::Key(KeyEvent {
                            code: KeyCode::Right,
                            ..
                        }) => {
                            direction = SearchDirection::Forward;
                            editor.move_cursor(Event::Key(KeyEvent {
                                code: KeyCode::Right,
                                modifiers: KeyModifiers::NONE,
                            }));
                            moved = true;
                        }
                        Event::Key(KeyEvent {
                            code: KeyCode::Char('p'),
                            ..
                        })
                        | Event::Key(KeyEvent {
                            code: KeyCode::Left,
                            ..
                        }) => direction = SearchDirection::Backward,
                        _ => direction = SearchDirection::Forward,
                    }
                    if let Some(position) =
                        editor
                            .document
                            .find(&query, &editor.cursor_position, direction)
                    {
                        editor.cursor_position = position;
                        editor.scroll();
                    } else if moved {
                        editor.move_cursor(Event::Key(KeyEvent {
                            code: KeyCode::Left,
                            modifiers: KeyModifiers::NONE,
                        }));
                    }
                    editor.highlighted_word = Some(query.to_string());
                },
            )
            .unwrap_or(None);

        if query.is_none() {
            self.cursor_position = old_position;
            self.scroll();
        }
        self.highlighted_word = None;
    }
    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let event = Terminal::read_key()?;
        match (&self.mode, event) {
            // go to visual mode when Ctrl-V is pressed in normal mode
            (
                Mode::Normal,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('v'),
                    modifiers: KeyModifiers::CONTROL,
                }),
            ) => self.mode = Mode::Visual,

            // go to normal mode when Esc is pressed in Insert or Visual Mode
            (
                _,
                Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                }),
            ) => self.mode = Mode::Normal,

            // go to insert mode when i is pressed.
            (
                Mode::Normal,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('i'),
                    ..
                }),
            ) => {
                self.mode = Mode::Insert;
                Terminal::cursor_hide();
            }

            // go to insert mode one past cursor if a is pressed.
            (
                Mode::Normal,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('a'),
                    ..
                }),
            ) => {
                self.move_cursor(Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    modifiers: KeyModifiers::NONE,
                }));
                self.mode = Mode::Insert;
            }

            // go to insert mode at end of line if A is pressed.
            (
                Mode::Normal,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('A'),
                    ..
                }),
            ) => {
                self.cursor_position.x = self
                    .document
                    .row(self.cursor_position.y)
                    .unwrap_or(&Row::default())
                    .len();
                self.mode = Mode::Insert;
            }

                        (
                Mode::Normal,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('b'),
                    ..
                }),
            ) => {
                let width = self.terminal.size().width as usize;
                let height = self.terminal.size().height as usize;

                // keep moving right until you've seen both a space and a char.
                let mut seen_char = false;
                let mut seen_space = false;
                let mut i = 0;
                while i < 500 {
                    if seen_char == true && seen_space == true {
                        break;
                    }
                    let row = self.document.row(self.cursor_position.y);
                    if row.is_some() {
                        if let Some(c) = row.unwrap().get(self.cursor_position.x) {
                            match c {
                                " " | "\t" | "\n" => seen_space = true,
                                _ => seen_char = true,
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }

                    self.move_cursor(Event::Key(KeyEvent {
                        code: KeyCode::Left,
                        modifiers: KeyModifiers::NONE,
                    }));
                    i += 1;
                }
            }

            // either save if :w or go find next word.
            // FIXME: Broken
            (
                Mode::Normal,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('w'),
                    ..
                }),
            ) => {
                // move cursor to the left until the character underneath is not a space?
                if self.previous_characters.last() != Some(&':') {
                    let width = self.terminal.size().width as usize;
                    let height = self.terminal.size().height as usize;

                    // keep moving right until you've seen both a space and a char.
                    let mut seen_char = false;
                    let mut seen_space = false;
                    let mut i = 0;
                    while i < 500 {
                        if seen_char == true && seen_space == true {
                            break;
                        }
                        let row = self.document.row(self.cursor_position.y);
                        if row.is_some() {
                            if let Some(c) = row.unwrap().get(self.cursor_position.x) {
                                match c {
                                    " " | "\t" | "\n" => seen_space = true,
                                    _ => seen_char = true,
                                }
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }

                        self.move_cursor(Event::Key(KeyEvent {
                            code: KeyCode::Right,
                            modifiers: KeyModifiers::NONE,
                        }));
                        i += 1;
                    }
                }

                // Save with :w in normal mode.
                if self.previous_characters.last() == Some(&':') {
                    self.save();
                    self.previous_characters.clear();
                }
            }

            // move around in normal and visual mode with h | l | j | k | Up | Down | Left | Right
            (
                Mode::Normal | Mode::Visual,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('h' | 'l' | 'j' | 'k'),
                    ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Up, ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Left,
                    ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    ..
                }),
            ) => self.move_cursor(event),
            // delete under cursor with x
            (
                Mode::Normal,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('x'),
                    ..
                }),
            ) => {
                self.document.delete(&self.cursor_position);
                self.move_cursor(Event::Key(KeyEvent {
                    code: KeyCode::Left,
                    modifiers: KeyModifiers::NONE,
                }))
            }

            // delete line with 'D'
            (
                Mode::Normal,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('D'),
                    ..
                }),
            ) => self.document.delete_line(self.cursor_position.y),

            // delete line with 'dd'
            (
                Mode::Normal,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('d'),
                    modifiers: KeyModifiers::NONE,
                }),
            ) => {
                if self.previous_characters.last() == Some(&'d') {
                    self.document.delete_line(self.cursor_position.y);
                    self.previous_characters.clear();
                } else {
                    self.previous_characters.push('d');
                }
            }

            // Quit with ':q'
            (
                Mode::Normal,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    ..
                }),
            ) => {
                if self.previous_characters.last() == Some(&':') {
                    if self.document.is_dirty() {
                        self.status_message =
                            StatusMessage::from("WARNING! File has unsaved changes.");
                        return Ok(());
                    }
                    self.should_quit = true;
                }
            }

            // Quit without saving with :!
            (
                Mode::Normal,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('!'),
                    ..
                }),
            ) => {
                if self.previous_characters.last() == Some(&':') {
                    self.should_quit = true;
                }
            }

            // insert newline after cursor with o
            (
                Mode::Normal,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('o'),
                    ..
                }),
            ) => {
                let new_position = &mut self.cursor_position;
                new_position.y = new_position.y.saturating_add(1);
                new_position.x = 0;
                self.document.insert_newline(new_position);
                self.mode = Mode::Insert;
            }

            // insert newline before with O
            (
                Mode::Normal,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('O'),
                    ..
                }),
            ) => {
                let new_position = &mut self.cursor_position;
                new_position.y = new_position.y.saturating_sub(1);
                new_position.x = 0;
                self.document.insert_newline(new_position);
                self.mode = Mode::Insert;
            }

            // Enter / to search in normal mode.
            (
                Mode::Normal,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('/'),
                    modifiers: KeyModifiers::NONE,
                }),
            ) => self.search(),

            // Enter Backspace in Insert mode to delete a char.
            (
                Mode::Insert,
                Event::Key(KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                }),
            ) => {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    self.move_cursor(Event::Key(KeyEvent {
                        code: KeyCode::Left,
                        modifiers: KeyModifiers::NONE,
                    }));
                    self.document.delete(&self.cursor_position);
                }
            }

            // Go to the end of the line with $
            (
                Mode::Normal,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('$'),
                    ..
                }),
            ) => {
                self.cursor_position.x = self
                    .document
                    .row(self.cursor_position.y)
                    .unwrap_or(&Row::default())
                    .len();
            }

            // Go to the beginning of the line with ^
            (
                Mode::Normal,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('^'),
                    ..
                }),
            ) => {
                self.cursor_position.x = 0;
            }

            // Insert if a char is pressed in Insert mode.
            (
                Mode::Insert,
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                }),
            ) => {
                self.document.insert(&self.cursor_position, c);
                self.move_cursor(Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    modifiers: KeyModifiers::NONE,
                }));
            }

            // Insert a newline when Enter is pressed.
            (
                Mode::Insert,
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }),
            ) => {
                self.document.insert(&self.cursor_position, '\n');
                self.move_cursor(Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    modifiers: KeyModifiers::NONE,
                }));
            }

            // Go to top of document with 'gg'
            (
                Mode::Normal,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('g'),
                    ..
                }),
            ) => {
                if self.previous_characters.last() == Some(&'g') {
                    self.cursor_position.y = 0;
                    self.previous_characters.clear();
                } else {
                    let mut position = 0;
                    let mut digit = 0;
                    while let Some(c) = self.previous_characters.pop() {
                        if c.is_numeric() {
                            position += (10_usize.pow(digit)) * (c.to_digit(10).unwrap() as usize);
                            digit += 1;
                        } else {
                            break;
                        }
                    }
                    if position == 0 {
                        self.previous_characters.push('g');
                    } else {
                        if position > self.document.len() - 1 {
                            self.cursor_position.y = self.document.len() - 1;
                        } else {
                            self.cursor_position.y = position;
                        }
                        self.previous_characters.clear();
                    }
                }
            }

            // Go to bottom of document with 'G'
            (
                Mode::Normal,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('G'),
                    ..
                }),
            ) => {
                self.cursor_position.y = self.document.len() - 1;
            }

            // push char to vector in normal mode if no use for it.
            (
                Mode::Normal,
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                }),
            ) => self.previous_characters.push(c),
            _ => (),
        }
        self.scroll();
        Ok(())
    }
    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let mut offset = &mut self.offset;
        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }
        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }
    fn move_cursor(&mut self, event: Event) {
        let Position { mut y, mut x } = self.cursor_position;
        let height = self.document.len();
        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('k'),
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Up, ..
            }) => {
                let mut position = 0;
                let mut digit = 0;
                while let Some(c) = self.previous_characters.pop() {
                    if c.is_numeric() {
                        position += (10_usize.pow(digit)) * (c.to_digit(10).unwrap() as usize);
                        digit += 1;
                    } else {
                        break;
                    }
                }
                if position > y {
                    y = 0;
                } else if position == 0 {
                    if y == 0 {
                        y = 0;
                    } else {
                        y -= 1;
                    }
                } else {
                    y -= position;
                }
                self.previous_characters.clear();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Down,
                ..
            }) => {
                let mut position = 0;
                let mut digit = 0;
                while let Some(c) = self.previous_characters.pop() {
                    if c.is_numeric() {
                        position += (10_usize.pow(digit)) * (c.to_digit(10).unwrap() as usize);
                        digit += 1;
                    } else {
                        break;
                    }
                }
                if y + position > self.document.len() - 1 {
                    y = self.document.len() - 1;
                } else if position == 0 {
                    if y + 1 > self.document.len() - 1 {
                        y = self.document.len() - 1;
                    } else {
                        y += 1;
                    }
                } else {
                    y += position;
                }
                self.previous_characters.clear();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('h'),
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Left,
                ..
            }) => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    if let Some(row) = self.document.row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    }
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('l'),
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Right,
                ..
            }) => {
                if x < width {
                    x += 1;
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            }
            _ => (),
        }
        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        if x > width {
            x = width;
        }

        self.cursor_position = Position { x, y }
    }
    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("rvim -- version {}", VERSION);
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        #[allow(clippy::integer_arithmetic, clippy::integer_division)]
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }
    pub fn draw_row(&self, row: &Row, row_number: u16) {
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let start = self.offset.x;
        let end = self.offset.x.saturating_add(width);
        let current_line_number = self.cursor_position.y.saturating_add(1);
        let relative_position = height.saturating_sub(height.saturating_sub(row_number.into())) as usize;
        let row_number = row_number as usize;
        let fold_number = self.cursor_position.y.saturating_div(height as usize);
        let cursor_row = self.cursor_row();
        // current line number = where the cursor is
        // calculate the offset of the cursor
        // if it's the next line, add a line, else don't

        //  -------------------------------------------------
        //  | cursor: 0 | row: 1 | cursor_row: 2 | height: 4 |
        //  | cursor: 1 | row: 2 | cursor_row: 3 | height: 4 |
        //  | cursor: 2 | row: 3 | cursor_row: 4 | height: 4 |
        //  | cursor: 3 | row: 4 | cursor_row: 5 | height: 4 |
        //  ------------|--------|-------------- |-----------|
        //->| cursor: 4 | row: 5 | cursor_row: 2 | height: 4 |
        //  | cursor: 5 | row: 6 | cursor_row: 3 | height: 4 |
        //  | cursor: 6 | row: 7 | cursor_row: 4 | height: 4 |
        //  | cursor: 7 | row: 8 | cursor_row: 5 | height: 4 |
        //  -------------------------------------------------
        let line_no = relative_position.saturating_add(0);
        // relative position goes from 56 to 1.

        let row = row.render(start, end, line_no);

        println!("{}\r", row)
    }
    fn cursor_row(&self) -> usize {
        self.cursor_position.y % (self.terminal.size().height as usize)
    }
    #[allow(clippy::integer_division, clippy::integer_arithmetic)]
    fn draw_rows(&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0..height {
            Terminal::clear_current_line();
            if let Some(row) = self
                .document
                .row(self.offset.y.saturating_add(terminal_row as usize))
            {
                self.draw_row(row, terminal_row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }
        }
    }
    fn draw_status_bar(&self) {
        let mut status;
        let width = self.terminal.size().width as usize;
        let modified_indicator = if self.document.is_dirty() {
            " (modified)"
        } else {
            ""
        };

        let mut file_name = "[No Name]".to_string();
        if let Some(name) = &self.document.file_name {
            file_name = name.clone();
            file_name.truncate(20);
        }
        status = format!(
            "{} - {} lines{}",
            file_name,
            self.document.len(),
            modified_indicator
        );

        let line_indicator = format!(
            "{}: {} | {}:{}",
            self.mode,
            self.document.file_type(),
            self.cursor_position.y.saturating_add(1),
            self.cursor_position.x.saturating_add(1),
        );
        #[allow(clippy::integer_arithmetic)]
        let len = status.len() + line_indicator.len();
        status.push_str(&" ".repeat(width.saturating_sub(len.saturating_add(5))));
        status = format!("{}{}", status, line_indicator);
        status.truncate(width);
        Terminal::set_bg_color(STATUS_BG_COLOR);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        println!("{}\r", status);
        Terminal::reset_fg_color();
        Terminal::reset_bg_color();
    }
    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        let message = &self.status_message;
        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            print!("{}", text);
        }
    }
    fn prompt<C>(&mut self, prompt: &str, mut callback: C) -> Result<Option<String>, std::io::Error>
    where
        C: FnMut(&mut Self, Event, &String),
    {
        let mut result = String::new();
        loop {
            self.status_message = StatusMessage::from(format!("{}{}", prompt, result));
            self.refresh_screen()?;
            let key = Terminal::read_key()?;
            match key {
                Event::Key(KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                }) => result.truncate(result.len().saturating_sub(1)),
                Event::Key(KeyEvent {
                    code: KeyCode::Char('\n'),
                    ..
                }) => break,
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                }) => {
                    if !c.is_control() {
                        result.push(c);
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                }) => {
                    result.truncate(0);
                    break;
                }
                _ => (),
            }
            callback(self, key, &result);
        }
        self.status_message = StatusMessage::from(String::new());
        if result.is_empty() {
            return Ok(None);
        }
        Ok(Some(result))
    }
}

fn die(e: std::io::Error) {
    std::panic::panic_any(e);
}
