///external crate
use std::{env, io};
use termion::{event::Key, input::TermRead};

/// 处理器类型
#[derive(Default)]
pub struct Processor;

impl Processor {
    /// 处理按键:函数接受闭包, 该闭包用于处理按键
    pub fn process_keypress<F>(&self, call: F) -> Result<(), std::io::Error>
    where
        F: FnOnce(Key) -> (),
    {
        let pressed_key = Processor::read_key()?;
        Ok(call(pressed_key))
    }

    /// 读取按键
    pub fn read_key() -> Result<Key, io::Error> {
        loop {
            if let Some(key) = io::stdin().lock().keys().next() {
                return key;
            }
        }
    }

    /// 读取命令行参数
    pub fn read_filename_for_command() -> Option<String> {
        let args: Vec<String> = env::args().collect();
        Some(String::from(args.get(1)?))
    }
}
