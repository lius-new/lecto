pub struct Row {
    text: String,
}

impl From<&str> for Row {
    fn from(value: &str) -> Self {
        Self {
            text: String::from(value),
        }
    }
}
impl Row {
    /// 渲染文本
    /// end: 实际值如果大于终端长度,那么就用终端长度. 否则就是实际值(防止值超出范围)
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = std::cmp::min(end, self.text.len());
        let start = std::cmp::min(start, end);
        self.text.get(start..end).unwrap_or_default().to_string()
    }
}
