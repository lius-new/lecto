///internal crate
use super::{
    constants,
    document::Document,
    processor::Processor,
    row::Row,
    terminal::Terminal,
    utils::{self, die},
};
///external crate
use std::{cell::RefCell, io::stdout, usize};
use termion::{event::Key, raw::IntoRawMode};

/// 编辑器中光标位置
#[derive(Default, Clone, Copy, Debug)]
pub struct Position {
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
    cursor_position: RefCell<Position>, // 光标位置, 读文件时是光标在文本中的位置
    offset: RefCell<Position>,          // 读文件时文本在窗口中文档的偏移量
    terminal: Terminal,
    processor: Processor,
    document: Document,
}
impl Default for Editor {
    fn default() -> Self {
        // 通过是否存在filename参数来构建不同的Document实例
        let document = if let Some(filename) = Processor::read_filename_for_command() {
            Document::open(&filename).unwrap_or_default()
        } else {
            Document::default()
        };

        Self {
            should_quit: RefCell::new(false),
            show_welcome: RefCell::new(true),
            cursor_position: RefCell::new(Position::default()),
            offset: RefCell::new(Position::default()),
            terminal: Terminal::default(),
            processor: Processor::default(),
            document,
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
            _ => (),
        }
        self.scroll()
    }

    /// 刷新文本编辑器屏幕
    pub fn refresh_editor_screen(&self) -> Result<(), std::io::Error> {
        self.terminal.cursor_hide();
        self.terminal.cursor_position(&Position::default());
        self.draw_start_running_symbol();
        // 设置光标的位置, 此时光标和文本偏移绑定到一起了.
        // 假设正如scroll方法描述, 当向下碰到边界并越过时(第一次碰到)，此时y = 8, offset = 1,所以光标为 8 -1 = 7, 此时光标就是为最后一行位置. 如果一直就y=9 offset = 2,光标7,...
        // 假设正如scroll方法描述, 当向上碰到边界并越过时，此时y = 0, offset = 1,所以光标为 0 - 1 = 0 (saturating_sub), 此时光标是为第一行位置. 如果一直就y=0 offset = 0,光标0,...
        self.terminal.cursor_position(&self.get_cursor_position());
        self.terminal.cursor_show();
        self.terminal.flush()
    }

    /// 如果是文档，那么绘制文档行(这里文档指文件)
    /// 渲染文本的宽度(第一个字符=(offset.x=0)): 即start=0,end=0+width,渲染文本是render(0,width)=>text[0,min(width,text_len)], 得出结论: 要么终端长度要么文本长度. 正常显示
    /// 渲染文本的宽度(第五个字符=(offset.x=4)): 即start=4,end=4+width,渲染文本是render(4,4+width)=>text[4,min(4+width,text_len)],
    pub fn draw_document_row(&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let x = self.get_offset().x;
        let start = x;
        let end = x + width;
        let row = row.render(start, end);
        self.terminal.draw_row(&row);
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

            // 如果有绘制内容就绘制(每次移动窗口内容都会重新绘制. 实际上只有offset.y发生变化才会导致现实的内容的重新绘制)
            // 绘制的内容是当前终端位置(窗口位置)加上光标所在文本的位置(显示位置). 比如开始状态下都在1,
            // 那么从上至下正常显示。1-1，2-2，3-3=>窗口第一行显示文本第一行,窗口第二行显示文本第二行,...
            // 当向下移动到第8行时为: 1-8，2-9，3-10=>窗口第一行显示文本第八行,窗口第二行显示文本第九行,...
            // 当向上移动到第7行时为: 1-7，2-8，3-9=>窗口第一行显示文本第七行,窗口第二行显示文本第八行,...
            if let Some(row) = self
                .document
                .row(terminal_row as usize + self.get_offset().y)
            {
                self.draw_document_row(row);
            } else if terminal_row == height / 2
                && self.get_show_welcome()
                && self.document.is_empty()
            {
                self.draw_welcome_message();
            } else {
                self.terminal.draw_row("~");
            }
        }
    }

    /// 文档上下翻动
    /// 修改文本的偏移量从而实现在窗口位置的变化
    /// cursor_position表示光标在文本的位置
    /// width,height表示终端(可视)大小
    /// offset: 表示文本偏移量,默认是0, 因为调用的是default()
    /// 假设光标开始向下移动, 当y = 1时, 对于y的判断条件(if 1 < 0 & else if 1 > 8)均不成立,且此后移动7次光标都不会导致文本显示变化(此处假设height=8)
    /// 假设光标开始向下移动, 当y = 8时, 对于y的判断条件(if 8 < 0 & else if 8 > 8)后者成立,此时窗口中的内容就会改变, 因为offset.y改变(y - height + 1 = 1)了. 那么导致draw_rows方法改变.
    /// 假设光标开始向上移动, 当y = 7时, 对于y的判断条件(if 7 < 1 & else if 7 > 8)均不成立,且此后移动7次光标都不会导致文本显示变化(此处假设height=8)
    /// 假设光标开始向上移动, 当y = 0时, 对于y的判断条件(if 0 < 1 & else if 0 > 8)前者成立,此时窗口中的内容就会改变, 因为offset.y改变(y = offset.y = 0)了. 那么导致draw_rows方法改变.
    /// 那么实际上只有当到最上面一行或最下面一行再进行往上面移动或者往下面移动才会导致offset.y发生变化
    pub fn scroll(&self) {
        let Position { x, y } = self.get_cursor_position();
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let mut offset = self.offset.borrow_mut();

        if y < offset.y {
            offset.set_position_y(y);
        } else if y >= offset.y.saturating_add(height) {
            offset.set_position_y(y.saturating_sub(height).saturating_add(1));
        }

        if x < offset.x {
            offset.set_position_x(x);
        } else if x >= offset.x.saturating_add(width) {
            offset.set_position_x(x.saturating_sub(width).saturating_add(1));
        }
    }

    /// 移动光标
    pub fn move_cursor(&self, key: Key) {
        let mut cursor_postion = self.cursor_position.borrow_mut();
        let (x, y) = (cursor_postion.x, cursor_postion.y);

        let height = if !self.document.is_empty() {
            self.document.len()
        } else {
            self.terminal.size().height.saturating_sub(1) as usize
        };
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
    /// 获取offset(偏移量)
    fn get_offset(&self) -> Position {
        *self.offset.borrow()
    }
}
