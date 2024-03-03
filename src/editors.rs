use crate::{document::Document, terminal::Size, Row, Terminal};
use core::panic;
use std::time::{Duration, Instant};
use std::{
    env,
    io::{self},
    usize,
};
use termion::{color, event::Key};

const STATUS_FG_COLOR: color::Rgb = color::Rgb(63, 63, 63);
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239);
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}
struct StatusMessgae {
    text: String,
    time: Instant,
}

impl StatusMessgae {
    fn from(message: String) -> Self {
        Self {
            time: Instant::now(),
            text: message,
        }
    }
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position, // 光标位置, 读文件时是光标在文本中的位置
    offset: Position,          // 读文件时文本在窗口中的偏移量
    document: Document,
    status_message: StatusMessgae,
}

impl Editor {
    /// 启动编辑器
    pub fn run(&mut self) {
        // 监听每次输入并处理输入(是否出错)
        loop {
            if let Err(err) = self.refresh_screen() {
                die(err)
            }
            if self.should_quit {
                break;
            }
            if let Err(err) = self.process_keypress() {
                die(err)
            }
        }
    }

    /// 处理每次输入的key
    pub fn process_keypress(&mut self) -> Result<(), io::Error> {
        let pressed_key = Terminal::read_key()?;
        match pressed_key {
            Key::Ctrl('q') => self.should_quit = true,
            Key::Ctrl('s') => self.save(),
            Key::Char(c) => {
                self.document.insert(&self.cursor_position, c);
                self.move_cursor(Key::Right); // 插入字符后使光标向右移动
            }
            Key::Backspace | Key::Delete => {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    self.move_cursor(Key::Left);
                    self.document.delete(&self.cursor_position)
                }
            }
            Key::Up
            | Key::Down
            | Key::Left
            | Key::Right
            | Key::PageUp
            | Key::PageDown
            | Key::End
            | Key::Home => self.move_cursor(pressed_key),
            _ => (),
        }
        self.scroll();
        Ok(())
    }

    /// 保存文件
    fn save(&mut self) {
        if self.document.file_name.is_none() {
            let new_name = self.prompt("Save as: ").unwrap_or(Option::None);
            if new_name.is_none() {
                self.status_message = StatusMessgae::from("Save aborted.".to_string());
                return;
            }
            self.document.file_name = new_name;
        }

        if self.document.save().is_ok() {
            self.status_message = StatusMessgae::from("File saved successfuly.".to_string())
        } else {
            self.status_message = StatusMessgae::from("Error writing file!".to_string());
        }
    }

    fn prompt(&mut self, prompt: &str) -> Result<Option<String>, io::Error> {
        let mut result = String::new();
        loop {
            self.status_message = StatusMessgae::from(format!("{}{}", prompt, result));
            self.refresh_screen()?;

            match Terminal::read_key()? {
                Key::Backspace => {
                    if !result.is_empty() {
                        result.truncate(result.len() - 1)
                    }
                }
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
        }

        self.status_message = StatusMessgae::from(String::new());
        if result.is_empty() {
            return Ok(None);
        }
        Ok(Some(result))
    }

    /// refresh terminal windows screen
    /// 运行时刷新屏幕, 刷新
    pub fn refresh_screen(&self) -> Result<(), io::Error> {
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position::default());
        if self.should_quit {
            Terminal::clear_screen();
            println!("GoodBye.\r");
        } else {
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            // 设置光标的位置, 此时光标和文本偏移绑定到一起了.
            // 假设正如scroll方法描述, 当向下碰到边界并越过时(第一次碰到)，此时y = 8, offset = 1,所以光标为 8 -1 = 7, 此时光标就是为最后一行位置. 如果一直就y=9 offset = 2,光标7,...
            // 假设正如scroll方法描述, 当向上碰到边界并越过时，此时y = 0, offset = 1,所以光标为 0 - 1 = 0 (saturating_sub), 此时光标是为第一行位置. 如果一直就y=0 offset = 0,光标0,...
            Terminal::cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }
        Terminal::cursor_show();
        Terminal::flush()
    }

    /// 移动光标
    pub fn move_cursor(&mut self, key: Key) -> () {
        let terminal_height = self.terminal.size().height as usize;
        let Position { mut x, mut y } = self.cursor_position;
        // 该行用于设置行边界(移动时判断). 最开始边界是terminal width, 现在的边界是文本长度或0
        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };

        let height = self.document.len();
        match key {
            Key::Up => y = y.saturating_sub(1),
            Key::Down => {
                if y < height {
                    y = y.saturating_add(1)
                }
            }
            Key::Left => {
                // 如果是正常移动那么就往前一个字节移动.如果是第一个字符，那么就退回到上一行的最后一个字节
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
            Key::Right => {
                // 如果是正常移动那么就向后一个字节移动,如果是最后一个字节那么就跳转到下一行并设置为第一个字节
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
            _ => (),
        }

        //  下方连续代码: 当前面设置了width, 此时向下移动(y轴), 那么下一行也许并没有这么长,那么x的值也该是这行的末尾，而非是和上一行一样长
        //  x > width: 如果当前位置大于文本长度(上行注释描述)
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

    /// 修改文本的偏移量从而实现在窗口位置的变化
    /// cursor_position表示光标在文本的位置
    /// width,height表示终端(可视)大小
    /// offset: 表示文本偏移量,默认是0, 因为调用的是default()
    /// 假设光标开始向下移动, 当y = 1时, 对于y的判断条件(if 1 < 0 & else if 1 > 8)均不成立,且此后移动7次光标都不会导致文本显示变化(此处假设height=8)
    /// 假设光标开始向下移动, 当y = 8时, 对于y的判断条件(if 8 < 0 & else if 8 > 8)后者成立,此时窗口中的内容就会改变, 因为offset.y改变(y - height + 1 = 1)了. 那么导致draw_rows方法改变.
    /// 假设光标开始向上移动, 当y = 7时, 对于y的判断条件(if 7 < 1 & else if 7 > 8)均不成立,且此后移动7次光标都不会导致文本显示变化(此处假设height=8)
    /// 假设光标开始向上移动, 当y = 0时, 对于y的判断条件(if 0 < 1 & else if 0 > 8)前者成立,此时窗口中的内容就会改变, 因为offset.y改变(y = offset.y = 0)了. 那么导致draw_rows方法改变.
    /// 那么实际上只有当到最上面一行或最下面一行再进行往上面移动或者往下面移动才会导致offset.y发生变化
    pub fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let Size { width, height } = self.terminal.size();
        let (width, height) = (*width as usize, *height as usize);
        let offset = &mut self.offset;
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

    fn draw_status_bar(&self) {
        let mut status;
        let width = self.terminal.size().width as usize;
        let mut file_name = "[No Name]".to_string();
        if let Some(name) = &self.document.file_name {
            file_name = name.clone();
            file_name.truncate(20);
        }
        status = format!("{} - {} lines", file_name, self.document.len());

        let line_indicator = format!(
            "{}/{}",
            self.cursor_position.y.saturating_add(1),
            self.document.len()
        );
        let len = status.len() + line_indicator.len();
        if width > len {
            status.push_str(&" ".repeat(width - len));
        }

        status = format!("{}{}", status, line_indicator);
        status.truncate(width);

        Terminal::set_fg_color(STATUS_FG_COLOR);
        Terminal::set_bg_color(STATUS_BG_COLOR);
        println!("{}\r", status);

        Terminal::reset_fg_color();
        Terminal::reset_bg_color();
    }

    fn draw_message_bar(&self) {
        Terminal::clear_currsor_line();
        let message = &self.status_message;
        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize)
        }
    }

    /// 进入时输入
    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("Lecto Editor -- Version {} \r", VERSION);
        let width = self.terminal.size().width as usize;
        // 左内边距, 终端宽度减去开始欢迎文本宽度除以2
        let left_padding = width.saturating_sub(welcome_message.len()) / 2;
        let spaces = " ".repeat(left_padding.saturating_sub(1));
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width); // 将最终字符设置为终端宽度(包含空白文本)
        println!("{}\r", &welcome_message);
    }

    /// 获取每行的宽度并生成字符串后渲染
    pub fn draw_row(&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x; // 当前光标在文件位置
        let end = start + width; // 当前显示内容的字符串截至位置
        let row_string = row.render(start, end);
        println!("{}\r", row_string) // 注意不要使print, 因为所有的输出都会在一行了。
    }

    /// 为每一行绘制内容
    pub fn draw_rows(&self) -> () {
        let height = self.terminal.size().height;

        for terminal_row in 0..height {
            Terminal::clear_currsor_line();

            // 如果有绘制内容就绘制(每次移动窗口内容都会重新绘制. 实际上只有offset.y发生变化才会导致现实的内容的重新绘制)
            // 绘制的内容是当前终端位置(窗口位置)加上光标所在文本的位置(显示位置). 比如开始状态下都在1,
            // 那么从上至下正常显示。1-1，2-2，3-3=>窗口第一行显示文本第一行,窗口第二行显示文本第二行,...
            // 当向下移动到第8行时为: 1-8，2-9，3-10=>窗口第一行显示文本第八行,窗口第二行显示文本第九行,...
            // 当向上移动到第7行时为: 1-7，2-8，3-9=>窗口第一行显示文本第七行,窗口第二行显示文本第八行,...
            if let Some(row) = self.document.row(terminal_row as usize + self.offset.y) {
                self.draw_row(row)
            } else if self.document.is_empty() && terminal_row == height / 2 {
                // 在document为空时在屏幕中间绘制欢迎标语
                self.draw_welcome_message();
            } else {
                // 每一行绘制~
                println!("~\r")
            }
        }
    }

    /// 构建Editor实例
    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status = String::from("HELP: Ctrl-S = save | Ctrl-Q = quit");

        let document = if args.len() > 1 {
            let file_name = &args[1];
            let doc = Document::open(&file_name);
            if doc.is_ok() {
                doc.unwrap()
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
            cursor_position: Position::default(),
            offset: Position::default(),
            document,
            status_message: StatusMessgae::from(initial_status),
        }
    }
}

/// panic error
fn die(e: std::io::Error) {
    Terminal::clear_screen();
    panic!("{}", e)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let y = 0u16;
        println!("{}", y.saturating_sub(1))
    }

    #[test]
    fn string_len() {
        dbg!("aaa".to_string().len());
        dbg!("äää".to_string().len());
        dbg!("y̆y̆y̆".to_string().len());
        dbg!("❤❤❤".to_string().len());
    }
}
