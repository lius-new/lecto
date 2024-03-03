///internal crate
use super::{
    constants,
    processor::Processor,
    terminal::Terminal,
    utils::{self, die},
};
///external crate
use std::{cell::RefCell, io::stdout};
use termion::{event::Key, raw::IntoRawMode};

/// 编辑器中光标位置
#[derive(Default, Clone, Copy, Debug)]
pub(crate) struct Position {
    pub(crate) x: usize,
    pub(crate) y: usize,
}

impl Position {
    pub fn set_position_x(&mut self, x: usize) {
        self.x = x;
    }
    pub fn set_position_y(&mut self, y: usize) {
        self.y = y;
    }
}

/// 编辑器类型
pub struct Editor {
    should_quit: RefCell<bool>,
    show_welcome: RefCell<bool>,
    cursor_position: RefCell<Position>,
    terminal: Terminal,
    processor: Processor,
}
impl Default for Editor {
    fn default() -> Self {
        Self {
            should_quit: RefCell::new(false),
            show_welcome: RefCell::new(true),
            cursor_position: RefCell::new(Position::default()),
            terminal: Terminal::default(),
            processor: Processor::default(),
        }
    }
}
impl Editor {
    /// 启动编辑器
    pub fn run(&mut self) {
        let _stdout = stdout().into_raw_mode().unwrap();

        loop {
            if let Err(err) = self.refresh_editor_screen() {
                self.terminal.clear_screen();
                die(err)
            }
            if self.get_should_quit() {
                self.terminal.clear_screen();
                println!("Goodbye. \r");
                break;
            }
            if let Err(err) = self
                .processor
                .process_keypress(|key| self.editor_processor(key))
            {
                utils::die(err)
            }
        }
    }
    /// 文本编辑器处理案件的函数
    fn editor_processor(&self, key: Key) {
        match key {
            Key::Ctrl('q') => self.set_should_quit(true),
            Key::Up
            | Key::Down
            | Key::Left
            | Key::Right
            | Key::PageUp
            | Key::PageDown
            | Key::End
            | Key::Home => self.move_cursor(key),
            _ => println!("{:?}\r", key),
        }
    }

    /// 刷新文本编辑器屏幕
    pub fn refresh_editor_screen(&self) -> Result<(), std::io::Error> {
        self.terminal.cursor_hide();
        self.terminal.cursor_position(&Position::default());
        self.draw_start_running_symbol();
        self.terminal.cursor_position(&self.get_cursor_position());
        self.terminal.cursor_show();
        self.terminal.flush()
    }

    /// 绘制开始描述
    pub fn draw_welcome_message(&self) {
        self.terminal
            .draw_row_text_center("~", "Lecto Editor & LiusNew");
        self.terminal
            .draw_row_text_center("~", "This is My Customer Editor By Rust");
        self.terminal.draw_row_text_center("~", "\r");
        self.terminal
            .draw_row_text_center("~", &format!("Version: {}", constants::VERSION));
        self.set_show_welcome(false);
    }

    /// 文本编辑器打开后或运行时新行 绘制波浪线
    pub fn draw_start_running_symbol(&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0..height - 1 {
            self.terminal.clear_current_line();
            if terminal_row == height / 2 && self.get_show_welcome() {
                self.draw_welcome_message();
            } else {
                self.terminal.draw_row("~");
            }
        }
    }
    /// 移动光标
    pub fn move_cursor(&self, key: Key) {
        let mut cursor_postion = self.cursor_position.borrow_mut();
        let (x, y) = (cursor_postion.x, cursor_postion.y);

        let height = self.terminal.size().height.saturating_sub(1) as usize;
        let width = self.terminal.size().width.saturating_sub(1) as usize;
        match key {
            Key::Up => cursor_postion.set_position_y(y.saturating_sub(1)),
            Key::Down => {
                if y < height {
                    cursor_postion.set_position_y(y.saturating_add(1))
                }
            }
            Key::Left => cursor_postion.set_position_x(x.saturating_sub(1)),
            Key::Right => {
                if x < width {
                    cursor_postion.set_position_x(x.saturating_add(1))
                }
            }
            Key::PageUp => cursor_postion.set_position_y(0),
            Key::PageDown => cursor_postion.set_position_y(height),
            Key::Home => cursor_postion.set_position_x(0),
            Key::End => cursor_postion.set_position_x(width),
            _ => (),
        }
    }

    fn get_show_welcome(&self) -> bool {
        *self.show_welcome.borrow()
    }
    fn set_show_welcome(&self, value: bool) {
        *(self.show_welcome.borrow_mut()) = value
    }
    /// 获取 should_quit
    fn get_should_quit(&self) -> bool {
        *self.should_quit.borrow()
    }
    /// 设置 should_quit
    fn set_should_quit(&self, value: bool) {
        *(self.should_quit.borrow_mut()) = value
    }
    /// 获取光标位置
    fn get_cursor_position(&self) -> Position {
        *self.cursor_position.borrow()
    }
}
