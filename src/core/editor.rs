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
use std::{
    cell::RefCell,
    io::stdout,
    time::{Duration, Instant},
    usize,
};
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

/// 编辑器类型
pub struct Editor {
    should_quit: RefCell<bool>,
    show_welcome: RefCell<bool>,
    cursor_position: RefCell<Position>, // 光标位置, 读文件时是光标在文本中的位置
    offset: RefCell<Position>,          // 读文件时文本在窗口中文档的偏移量
    terminal: Terminal,
    processor: Processor,
    document: RefCell<Document>,
    status_message: RefCell<StatusMessage>,
}
impl Default for Editor {
    fn default() -> Self {
        // 通过是否存在filename参数来构建不同的Document实例
        let mut initial_status = String::from("HELP: Ctrl-Q=quit");
        let document = if let Some(filename) = Processor::read_filename_for_command() {
            let doc = Document::open(&filename);
            if doc.is_ok() {
                doc.unwrap()
            } else {
                initial_status = format!("Err Cloud not open file:{}", filename);
                Document::default()
            }
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
            status_message: RefCell::new(StatusMessage::from(initial_status)),
            document: RefCell::new(document),
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
            Key::Ctrl('q') => self.exit(),
            Key::Ctrl('s') => self.save(),
            Key::Char(c) => {
                self.insert_chat_at_document(c);
                self.move_cursor(Key::Right)
            }
            Key::Backspace => {
                self.move_cursor(Key::Left);
                self.delete_chat_at_document()
            }
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
        self.draw_status_bar();
        self.draw_message_bar();
        // 设置光标的位置, 此时光标和文本偏移绑定到一起了.
        // 假设正如scroll方法描述, 当向下碰到边界并越过时(第一次碰到)，此时y = 8, offset = 1,所以光标为 8 -1 = 7, 此时光标就是为最后一行位置. 如果一直就y=9 offset = 2,光标7,...
        // 假设正如scroll方法描述, 当向上碰到边界并越过时，此时y = 0, offset = 1,所以光标为 0 - 1 = 0 (saturating_sub), 此时光标是为第一行位置. 如果一直就y=0 offset = 0,光标0,...
        let Position { x, y } = self.get_cursor_position();
        self.terminal.cursor_position(&Position {
            x: x.saturating_sub(self.get_offset().x),
            y: y.saturating_sub(self.get_offset().y),
        });
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
    /// 绘制状态栏
    fn draw_status_bar(&self) {
        let mut status;
        let width = self.terminal.size().width as usize;
        let document = self.document.borrow();
        let modified_indicator = if document.is_dirty() {
            "(modified)"
        } else {
            ""
        };

        let mut file_name = "[No Name]".to_string();
        if let Some(name) = &document.file_name {
            file_name = name.clone();
            file_name.truncate(20);
        }
        status = format!(
            "{} - {} lines{}",
            file_name,
            &document.len(),
            modified_indicator
        );
        let line_indicator = format!(
            "{}/{}",
            self.get_cursor_position().y.saturating_add(1),
            &document.len()
        );
        let len = status.len() + line_indicator.len();

        if width > len {
            status.push_str(&" ".repeat(width - len))
        }
        status = format!("{}{}", status, line_indicator);
        status.truncate(width);
        Terminal::set_bg_color(constants::STATUS_BG_COLOR);
        Terminal::set_fg_color(constants::STATUS_FG_COLOR);
        self.terminal.draw_row(&status);
        Terminal::reset_fg_color();
        Terminal::reset_bg_color();
    }

    /// 绘制消息提示
    fn draw_message_bar(&self) {
        self.terminal.clear_current_line();
        let message = self.status_message.borrow();
        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            self.terminal.draw_row(&text);
        }
    }
    /// 让用户根据提示输入内容
    fn prompt(&self, prompt: &str) -> Result<Option<String>, std::io::Error> {
        let mut result = String::new();
        loop {
            self.rewrite_status_message(&format!("{}{}", prompt, result));
            self.refresh_editor_screen()?;
            match Processor::read_key()? {
                Key::Backspace => {
                    if !result.is_empty() {
                        result.truncate(result.len() - 1)
                    }
                }
                Key::Char('\n') => break,
                Key::Char(c) => {
                    if !c.is_control() {
                        result.push(c)
                    }
                }
                Key::Esc => {
                    result.truncate(0);
                    break;
                }
                _ => (),
            }
        }
        self.rewrite_status_message("");
        if result.is_empty() {
            return Ok(None);
        }

        Ok(Some(result))
    }

    /// 退出
    fn exit(&self) {
        if self.document.borrow().is_dirty() {
            let exit_value = self.prompt("Exit(y/n): ").unwrap_or(None);
            if let Some(exit_value) = exit_value {
                if exit_value.to_lowercase() == "y" {
                    self.set_should_quit(true)
                }
            }
        } else {
            self.set_should_quit(true)
        }
    }

    /// 保存文件
    fn save(&self) {
        if self.get_documnet_filename().is_none() {
            // 获取名称, 默认为(None)
            let new_name = self.prompt("Save as: ").unwrap_or(None);
            if new_name.is_none() {
                self.rewrite_status_message("Save aborted.");
                return;
            }
            self.reset_document_filename(new_name)
        }
        // 输出保存信息
        let text = if self.save_document().is_ok() {
            "File saved successfully.".to_string()
        } else {
            "Error writing file!".to_string()
        };
        self.rewrite_status_message(&text)
    }

    /// 文本编辑器打开后或运行时新行 绘制波浪线
    pub fn draw_start_running_symbol(&self) {
        let height = self.terminal.size().height;
        let document = self.document.borrow();
        for terminal_row in 0..height {
            self.terminal.clear_current_line();

            // 如果有绘制内容就绘制(每次移动窗口内容都会重新绘制. 实际上只有offset.y发生变化才会导致现实的内容的重新绘制)
            // 绘制的内容是当前终端位置(窗口位置)加上光标所在文本的位置(显示位置). 比如开始状态下都在1,
            // 那么从上至下正常显示。1-1，2-2，3-3=>窗口第一行显示文本第一行,窗口第二行显示文本第二行,...
            // 当向下移动到第8行时为: 1-8，2-9，3-10=>窗口第一行显示文本第八行,窗口第二行显示文本第九行,...
            // 当向上移动到第7行时为: 1-7，2-8，3-9=>窗口第一行显示文本第七行,窗口第二行显示文本第八行,...
            if let Some(row) = document.row(terminal_row as usize + self.get_offset().y) {
                self.draw_document_row(row);
            } else if terminal_row == height / 2 && self.get_show_welcome() && document.is_empty() {
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
        let (mut x, mut y) = (cursor_postion.x, cursor_postion.y);
        let document = self.document.borrow();

        let height = if !document.is_empty() {
            document.len()
        } else {
            self.terminal.size().height.saturating_sub(1) as usize
        };
        let mut width = if let Some(row) = document.row(y) {
            row.len()
        } else {
            0
            // self.terminal.size().width.saturating_sub(1) as usize
        };
        let terminal_height = self.terminal.size().height as usize;
        match key {
            Key::Up => y = y.saturating_sub(1),
            Key::Down => {
                if y < height {
                    y = y.saturating_add(1)
                }
            }
            Key::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    if let Some(row) = document.row(y) {
                        x = row.len()
                    } else {
                        x = 0;
                    }
                }
            }
            Key::Right => {
                if x < width {
                    x += 1;
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            }
            Key::PageUp => {
                y = if y > terminal_height {
                    y - terminal_height
                } else {
                    0
                }
            }
            Key::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y + terminal_height as usize
                } else {
                    height
                }
            }
            Key::Home => x = 0,
            Key::End => x = width,
            _ => (),
        }

        width = if let Some(row) = document.row(y) {
            row.len()
        } else {
            0
        };
        if x > width {
            x = width;
        }
        cursor_postion.set_position_x(x);
        cursor_postion.set_position_y(y);
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
    /// 获取document中的文件名
    fn get_documnet_filename(&self) -> Option<String> {
        self.document.borrow().file_name.clone()
    }
    /// 修改document中的文件名
    fn reset_document_filename(&self, filename: Option<String>) {
        let mut document = self.document.borrow_mut();
        document.file_name = filename
    }
    fn save_document(&self) -> Result<(), std::io::Error> {
        self.document.borrow_mut().save()
    }
    /// 当前光标处插入字符
    fn insert_chat_at_document(&self, c: char) {
        let cursor_position = &self.get_cursor_position();
        self.document.borrow_mut().inesrt(cursor_position, c);
    }
    /// 当前光标处删除字符
    fn delete_chat_at_document(&self) {
        let cursor_position = &self.get_cursor_position();

        if cursor_position.x > 0 || cursor_position.y > 0 {
            self.document.borrow_mut().delete(cursor_position);
        }
    }

    /// 修改status_message
    fn rewrite_status_message(&self, text: &str) {
        let mut statue_message = self.status_message.borrow_mut();
        statue_message.text = String::from(text);
    }
}
