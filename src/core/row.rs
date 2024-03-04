use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct Row {
    text: String,
    len: usize,
}

impl From<&str> for Row {
    fn from(value: &str) -> Self {
        let mut row = Self {
            text: String::from(value),
            len: 0,
        };
        row.update_len();
        row
    }
}
impl Row {
    /// 渲染文本
    /// end: 实际值如果大于终端长度,那么就用终端长度. 否则就是实际值(防止值超出范围)
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = std::cmp::min(end, self.text.len());
        let start = std::cmp::min(start, end);
        self.text.get(start..end).unwrap_or_default().to_string();
        let mut result = String::new();
        // graphemes 表示字位(光标移动的最小单位)
        // skip表示跳过几个(实际上就是第几个开始)
        // take表示几个字位长度
        for grapheme in self.text[..].graphemes(true).skip(start).take(end - start) {
            if grapheme == "\t" {
                result.push_str("  ");
            } else {
                result.push_str(grapheme)
            }
        }
        result
    }
    /// 字符串字位长度
    pub fn len(&self) -> usize {
        self.text[..].graphemes(true).count()
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    fn update_len(&mut self) {
        self.len = self.text[..].graphemes(true).count()
    }
}
