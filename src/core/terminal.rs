use super::editor::Position;
/// external crate
use std::{
    io::{self, stdout, Write},
    usize,
};
use termion::{
    color::{self},
    raw::{IntoRawMode, RawTerminal},
};

#[derive(Debug)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub struct Terminal {
    size: Size,
    _stdout: RawTerminal<std::io::Stdout>,
}

impl Default for Terminal {
    fn default() -> Self {
        let (width, height) = termion::terminal_size().unwrap();
        Self {
            size: Size {
                width,
                height: height - 2,
            },
            _stdout: stdout().into_raw_mode().unwrap(),
        }
    }
}

impl Terminal {
    /// 移动光标
    pub fn cursor_position(&self, position: &Position) {
        let Position { mut x, mut y } = position;
        x = x.saturating_add(1);
        y = y.saturating_add(1);
        let x = x as u16;
        let y = y as u16;
        print!("{}", termion::cursor::Goto(x, y));
    }

    /// 清空终端屏幕内容
    pub fn clear_screen(&self) {
        print!("{}", termion::clear::All)
    }
    /// 清空当前行
    pub fn clear_current_line(&self) {
        print!("{}", termion::clear::CurrentLine)
    }

    /// 终端绘制字符串
    pub fn draw_row(&self, text: &str) {
        println!("{}\r", text)
    }
    /// 终端绘制字符串
    pub fn draw_row_text_center(&self, prefix: &str, text: &str) {
        let width = self.size().width as usize;
        let len = text.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        self.draw_row(&format!("{}{}{}", prefix, spaces, text))
    }

    /// 隐藏光标
    pub fn cursor_hide(&self) {
        print!("{}", termion::cursor::Hide)
    }
    /// 显示光标
    pub fn cursor_show(&self) {
        print!("{}", termion::cursor::Show)
    }

    /// 终端大小
    pub fn size(&self) -> &Size {
        &self.size
    }
    /// 刷新输出流
    pub fn flush(&self) -> Result<(), std::io::Error> {
        io::stdout().flush()
    }

    /// 设置背景颜色
    pub fn set_bg_color(color: color::Rgb) {
        print!("{}", color::Bg(color))
    }
    /// 重置背景颜色
    pub fn reset_bg_color() {
        print!("{}", color::Bg(color::Reset))
    }
    /// 设置前景颜色
    pub fn set_fg_color(color: color::Rgb) {
        print!("{}", color::Fg(color))
    }
    /// 重置前景颜色
    pub fn reset_fg_color() {
        print!("{}", color::Fg(color::Reset))
    }
}
