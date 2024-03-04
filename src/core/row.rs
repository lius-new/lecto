use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Default)]
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
    /// 插入字符
    pub fn insert(&mut self, at: usize, c: char) {
        if at >= self.len() {
            self.text.push(c);
        } else {
            // result 获取的是前面的at之前的字符, remainder是获取at之后的字符
            let mut result: String = self.text[..].graphemes(true).take(at).collect();
            let remainder: String = self.text[..].graphemes(true).skip(at).collect();
            result.push(c);
            result.push_str(&remainder);
            self.text = result;
        }
        self.update_len();
    }
    /// 删除字符
    /// at表示x坐标,len表示行长度
    /// 当x大于行长度时
    pub fn delete(&mut self, at: usize) {
        if at >= self.len() {
            return;
        } else {
            // result 获取的是前面的at之前的字符, remainder是获取at之后的字符
            let mut result: String = self.text[..].graphemes(true).take(at).collect();
            let remainder: String = self.text[..].graphemes(true).skip(at + 1).collect();
            result.push_str(&remainder);
            self.text = result;
        }
        self.update_len()
    }

    /// 分割字符串(回车创建一个新行)
    /// 如果是在行中间, 那么就将当前行分为两部分, 第一部分作为新的当前行, 另一部分作为新行返回出去
    /// 如果是在行皆为, 那么当前行不发生变化, 会有一个空的字符串作为新行返回出去
    pub fn split(&mut self, at: usize) -> Self {
        let begining: String = self.text[..].graphemes(true).take(at).collect();
        let remainder: String = self.text[..].graphemes(true).skip(at).collect();
        self.text = begining;
        self.update_len();
        Self::from(&remainder[..])
    }

    /// 追加新的行
    pub fn append(&mut self, new: &Self) {
        self.text = format!("{}{}", self.text, new.text);
        self.update_len()
    }

    /// 转换为字节流
    pub fn as_bytes(&self) -> &[u8] {
        self.text.as_bytes()
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
