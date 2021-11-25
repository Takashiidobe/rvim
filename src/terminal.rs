use crate::Position;
use crossterm::event::{read, Event};
use crossterm::style::{Color, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{size, Clear, ClearType};
use crossterm::{cursor, queue};
use std::io::{self, stdout, Write};

pub struct Size {
    pub width: u16,
    pub height: u16,
}
pub struct Terminal {
    size: Size,
}

impl Terminal {
    pub fn default() -> Result<Self, std::io::Error> {
        let (width, height) = size()?;
        let width = width.saturating_add(5);
        let height = height.saturating_sub(2);
        Ok(Self {
            size: Size { width, height },
        })
    }
    pub fn size(&self) -> &Size {
        &self.size
    }
    pub fn clear_screen() {
        queue!(stdout(), Clear(ClearType::All)).unwrap();
    }

    pub fn cursor_position(position: &Position) {
        let Position { mut x, y } = position;
        if *y != 0 {
            x = x.saturating_add(5);
        }
        queue!(stdout(), cursor::MoveTo(x as u16, *y as u16)).unwrap();
    }
    pub fn flush() -> Result<(), std::io::Error> {
        io::stdout().flush()
    }
    pub fn read_key() -> Result<Event, std::io::Error> {
        loop {
            return Ok(read().unwrap());
        }
    }
    pub fn cursor_hide() {
        queue!(stdout(), cursor::Hide).unwrap();
    }
    pub fn cursor_show() {
        queue!(stdout(), cursor::Show).unwrap();
    }
    pub fn clear_current_line() {
        queue!(stdout(), Clear(ClearType::CurrentLine)).unwrap();
    }
    pub fn set_bg_color(color: Color) {
        queue!(stdout(), SetBackgroundColor(color)).unwrap();
    }
    pub fn reset_bg_color() {
        queue!(stdout(), SetBackgroundColor(Color::Reset)).unwrap();
    }
    pub fn set_fg_color(color: Color) {
        queue!(stdout(), SetForegroundColor(color)).unwrap();
    }
    pub fn reset_fg_color() {
        queue!(stdout(), SetForegroundColor(Color::Reset)).unwrap();
    }
}
