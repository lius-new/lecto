use std::{cmp, usize};

use unicode_segmentation::UnicodeSegmentation;

/// Row
#[derive(Debug, Default)]
pub struct Row {
    string: String,
    len: usize,
}

impl From<&str> for Row {
    fn from(value: &str) -> Self {
        let mut row = Self {
            string: String::from(value),
            len: 0,
        };

        row.update_len();
        row
    }
}
impl Row {
    /// 生成每一行内容
    /// start: 0
    /// end: terminal width
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);
        // self.string.get(start..end).unwrap_or_default().to_string();
        let mut result = String::new();
        for grapheme in self.string[..]
            .graphemes(true)
            .skip(start)
            .take(end - start)
        {
            if grapheme == "\t" {
                result.push_str(" ")
            } else {
                result.push_str(grapheme)
            }
        }
        result
    }

    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn update_len(&mut self) {
        self.len = self.string[..].graphemes(true).count();
    }
    /// 在当前行插入字符
    /// 分两种情况判断，是大于当前行还是小于当前行字符数量
    pub fn insert(&mut self, at: usize, c: char) {
        if at >= self.len() {
            self.string.push(c);
        } else {
            let mut result: String = self.string[..].graphemes(true).take(at).collect();
            let remainder: String = self.string[..].graphemes(true).skip(at).collect();
            result.push(c);
            result.push_str(&remainder);
            self.string = result;
        }
        self.update_len()
    }

    /// 在当前行删除字符
    pub fn delete(&mut self, at: usize) {
        // 删除的字符不是当前范围
        if at >= self.len() {
            return;
        } else {
            let mut result: String = self.string[..].graphemes(true).take(at).collect();
            let remainder: String = self.string[..].graphemes(true).skip(at + 1).collect();
            result.push_str(&remainder);
            self.string = result;
        }
        self.update_len();
    }

    pub fn append(&mut self, new: &Self) {
        self.string = format!("{}{}", self.string, new.string);
        self.update_len();
    }
    pub fn split(&mut self, at: usize) -> Self {
        let beginning: String = self.string[..].graphemes(true).take(at).collect();
        let remainder: String = self.string[..].graphemes(true).skip(at).collect();
        self.string = beginning;
        self.update_len();
        Self::from(&remainder[..])
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.string.as_bytes()
    }
}
