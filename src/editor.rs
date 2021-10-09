use crate::Document;
use crate::Row;
use crate::Terminal;
use std::env;
use std::fmt;
use std::time::Duration;
use std::time::Instant;
use termion::color;
use termion::event::Key;

const STATUS_FG_COLOR: color::Rgb = color::Rgb(63, 63, 63);
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239);
const VERSION: &str = env!("CARGO_PKG_VERSION");
static SPACE_CHARS: &str = " \t\n\r";
static ALPHABETICAL_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

const QUIT_TIMES: u8 = 1;

#[derive(PartialEq, Copy, Clone)]
pub enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Default, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

struct StatusMessage {
    text: String,
    time: Instant,
}

impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            time: Instant::now(),
            text: message,
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
    quit_times: u8,
    previous_characters: Vec<char>,
}

impl Editor {
    pub fn run(&mut self) {
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
    }
    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status = String::from("HELP: `/` = find | `:w` = save | `:q` = quit");

        let document = if let Some(file_name) = args.get(1) {
            let doc = Document::open(file_name);
            if let Ok(doc) = doc {
                doc
            } else {
                initial_status = format!("ERR: Could not open file: {}", file_name);
                Document::default()
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
            quit_times: QUIT_TIMES,
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
            Terminal::cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
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
                        Key::Char('n') | Key::Right => {
                            direction = SearchDirection::Forward;
                            editor.move_cursor(Key::Right);
                            moved = true;
                        }
                        Key::Char('p') | Key::Left => direction = SearchDirection::Backward,
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
                        editor.move_cursor(Key::Left);
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
        let pressed_key = Terminal::read_key()?;
        match (&self.mode, pressed_key) {
            // go to visual mode when Ctrl-V is pressed in normal mode
            (Mode::Normal, Key::Ctrl('v')) => self.mode = Mode::Visual,

            // go to normal mode when Esc is pressed in Insert or Visual Mode
            (_, Key::Esc) => self.mode = Mode::Normal,

            // go to insert mode when i is pressed.
            (Mode::Normal, Key::Char('i')) => {
                self.mode = Mode::Insert;
                Terminal::cursor_hide();
            }

            // go to insert mode one past cursor if a is pressed.
            (Mode::Normal, Key::Char('a')) => {
                self.move_cursor(Key::Right);
                self.mode = Mode::Insert;
            }

            // go to insert mode at end of line if A is pressed.
            (Mode::Normal, Key::Char('A')) => {
                self.cursor_position.x = self
                    .document
                    .row(self.cursor_position.y)
                    .unwrap_or(&Row::default())
                    .len();
                self.mode = Mode::Insert;
            }

            // either save if :w or go find next word.
            // FIXME: Broken
            (Mode::Normal, Key::Char('w')) => {
                // move cursor to the left until the character underneath is not a space?
                if self.previous_characters.last() != Some(&':') {
                    // if on an alphabetical character, find the next space char
                    // if curr char is a space
                    // keep going until you find the next alphanumeric char.
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
                Key::Char('h' | 'l' | 'j' | 'k') | Key::Up | Key::Down | Key::Left | Key::Right,
            ) => self.move_cursor(pressed_key),
            // delete under cursor with x
            (Mode::Normal, Key::Char('x')) => {
                self.document.delete(&self.cursor_position);
                self.move_cursor(Key::Left)
            }

            // delete line with 'D'
            (Mode::Normal, Key::Char('D')) => self.document.delete_line(self.cursor_position.y),

            // delete line with 'dd'
            (Mode::Normal, Key::Char('d')) => {
                if self.previous_characters.last() == Some(&'d') {
                    self.document.delete_line(self.cursor_position.y);
                    self.previous_characters.clear();
                } else {
                    self.previous_characters.push('d');
                }
            }

            // Quit with ':q'
            (Mode::Normal, Key::Char('q')) => {
                if self.previous_characters.last() == Some(&':') {
                    if self.quit_times > 0 && self.document.is_dirty() {
                        self.status_message = StatusMessage::from(format!(
                            "WARNING! File has unsaved changes. Press q {} more times to quit.",
                            self.quit_times
                        ));
                        self.quit_times -= 1;
                        return Ok(());
                    }
                    self.should_quit = true;
                }
            }

            // insert newline after cursor with o
            (Mode::Normal, Key::Char('o')) => {
                let new_position = &mut self.cursor_position;
                new_position.y = new_position.y.saturating_add(1);
                new_position.x = 0;
                self.document.insert_newline(new_position);
                self.mode = Mode::Insert;
            }

            // insert newline before with O
            (Mode::Normal, Key::Char('O')) => {
                let new_position = &mut self.cursor_position;
                new_position.y = new_position.y.saturating_sub(1);
                new_position.x = 0;
                self.document.insert_newline(new_position);
                self.mode = Mode::Insert;
            }

            // Enter / to search in normal mode.
            (Mode::Normal, Key::Char('/')) => self.search(),

            // Enter Backspace in Insert mode to delete a char.
            (Mode::Insert, Key::Backspace) => {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    self.move_cursor(Key::Left);
                    self.document.delete(&self.cursor_position);
                }
            }

            // Insert if a char is pressed in Insert mode.
            (Mode::Insert, Key::Char(c)) => {
                self.document.insert(&self.cursor_position, c);
                self.move_cursor(Key::Right);
            }

            // Go to top of document with 'g'
            (Mode::Normal, Key::Char('g')) => {
                if self.previous_characters.last() == Some(&'g') {
                    self.cursor_position.y = 0;
                    self.previous_characters.clear();
                } else {
                    self.previous_characters.push('g');
                }
            }

            // Go to bottom of document with 'G'
            (Mode::Normal, Key::Char('G')) => {
                self.cursor_position.y = self.document.len() - 1;
            }

            // push char to vector in normal mode if no use for it.
            (Mode::Normal, Key::Char(c)) => self.previous_characters.push(c),
            _ => (),
        }
        self.scroll();
        if self.quit_times < QUIT_TIMES {
            self.quit_times = QUIT_TIMES;
            self.status_message = StatusMessage::from(String::new());
        }
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
    fn move_cursor(&mut self, key: Key) {
        let Position { mut y, mut x } = self.cursor_position;
        let height = self.document.len();
        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        match key {
            Key::Char('k') | Key::Up => y = y.saturating_sub(1),
            Key::Char('j') | Key::Down => {
                if y < height.saturating_sub(1) {
                    y += 1;
                }
            }
            Key::Char('h') | Key::Left => {
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
            Key::Char('l') | Key::Right => {
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
    pub fn draw_row(&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x.saturating_add(width);
        let row = row.render(start, end);
        println!("{}\r", row)
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
                self.draw_row(row);
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
            "{}: {} | {}/{}",
            self.mode,
            self.document.file_type(),
            self.cursor_position.y.saturating_add(1),
            self.document.len()
        );
        #[allow(clippy::integer_arithmetic)]
        let len = status.len() + line_indicator.len();
        status.push_str(&" ".repeat(width.saturating_sub(len)));
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
        C: FnMut(&mut Self, Key, &String),
    {
        let mut result = String::new();
        loop {
            self.status_message = StatusMessage::from(format!("{}{}", prompt, result));
            self.refresh_screen()?;
            let key = Terminal::read_key()?;
            match key {
                Key::Backspace => result.truncate(result.len().saturating_sub(1)),
                Key::Char('\n') => break,
                Key::Char(c) => {
                    if !c.is_control() {
                        result.push(c);
                    }
                }
                Key::Esc => {
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
