use std::io::{self, stdout, Stdout, Write};
use termion::{
    color::{self},
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
};

use crate::Position;

/// size struct
pub struct Size {
    pub width: u16,
    pub height: u16,
}

/// terminal struct
pub struct Terminal {
    size: Size,
    _stdout: RawTerminal<Stdout>,
}

impl Terminal {
    /// Terminal Builder
    pub fn default() -> Result<Self, std::io::Error> {
        let size = termion::terminal_size()?;

        Ok(Self {
            size: Size {
                width: size.0,
                height: size.1.saturating_sub(2),
            },
            _stdout: stdout().into_raw_mode()?,
        })
    }

    /// Terminal Clear
    pub fn clear_screen() -> () {
        print!("{}", termion::clear::All);
    }

    /// Terminal cursor goto position
    pub fn cursor_position(position: &Position) {
        let Position { mut x, mut y } = position;
        x = x.saturating_add(1);
        y = y.saturating_add(1);
        let x = x as u16;
        let y = y as u16;
        print!("{}", termion::cursor::Goto(x, y));
    }

    /// Terminal flush
    pub fn flush() -> Result<(), io::Error> {
        io::stdout().flush()
    }

    /// Terminal cursor hide
    pub fn cursor_hide() {
        print!("{}", termion::cursor::Hide)
    }

    /// Terminal cursor show
    pub fn cursor_show() {
        print!("{}", termion::cursor::Show)
    }

    /// setting font color
    pub fn set_fg_color(color: color::Rgb) {
        print!("{}", color::Fg(color))
    }

    /// reset font color
    pub fn reset_fg_color() {
        print!("{}", color::Fg(color::Reset))
    }

    /// setting background
    pub fn set_bg_color(color: color::Rgb) {
        print!("{}", color::Bg(color))
    }

    /// reset  background
    pub fn reset_bg_color() {
        print!("{}", color::Bg(color::Reset))
    }

    /// Terminal clear cursor show
    pub fn clear_currsor_line() {
        print!("{}", termion::clear::CurrentLine)
    }

    /// Terminal Read stdout
    /// 开启循环, 每次循环等待输入, 输入成功后就返回key(输入值)
    pub fn read_key() -> Result<Key, io::Error> {
        loop {
            if let Some(key) = io::stdin().lock().keys().next() {
                return key;
            }
        }
    }

    /// Terminal Size Getter
    pub fn size(&self) -> &Size {
        &self.size
    }
}
