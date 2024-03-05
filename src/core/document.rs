use std::{
    fs,
    io::{Error, Write},
};

use super::{editor::Position, row::Row};

#[derive(Default, Debug)]
pub struct Document {
    rows: Vec<Row>,
    dirty: bool,
    pub file_name: Option<String>,
}

impl Document {
    /// 打开一个文档
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(filename)?;
        let mut rows = Vec::new();

        for value in contents.lines() {
            rows.push(Row::from(value));
        }

        Ok(Self {
            rows,
            file_name: Some(filename.to_string()),
            dirty: false,
        })
    }
    /// 插入新行
    pub fn insert_new_line(&mut self, at: &Position) {
        // 如果是最后一行直接追加
        if at.y == self.len() {
            self.rows.push(Row::default());
            return;
        }

        let new_row = self.rows.get_mut(at.y).unwrap().split(at.x);
        self.rows.insert(at.y + 1, new_row)
    }

    /// 插入字符
    pub fn inesrt(&mut self, at: &Position, c: char) {
        // 大于文档长度
        if at.y > self.len() {
            return;
        }
        self.dirty = true;

        if c == '\n' {
            self.insert_new_line(at);
            return;
        }
        if at.y == self.len() {
            let mut row = Row::default();
            row.insert(0, c);
            self.rows.push(row)
        } else {
            let row = self.rows.get_mut(at.y).unwrap();
            row.insert(at.x, c);
        }
    }
    /// 删除字符
    pub fn delete(&mut self, at: &Position) {
        let len = self.len();

        // 大于文档长度, 不在删除的范围内
        if at.y >= len {
            return;
        }
        self.dirty = true;

        // 光标在x轴是最后一个字符以及光标不是最后一行(光标是否在一行末尾及是否是最后一个字符 --> 行为空(x:0,len:0))
        // 事实上删除前会向前移动, 此时如果行首, 那么会跳到前一行
        // 假设前一行是空行且当前在{x:0,y:5,len:10}, 此时删除前会光标会是{x:0,y:4,len:0}, 此时下一行就会移动到上一行去
        // 假设前一行不是空行(len=8)且当前在{x:0,y:5,len:10},此时删除光标为{x:8(上一行的末尾位置等于长度),y:4,len:8}, 此时下一行也会移动到上一行去
        if at.x == self.rows.get_mut(at.y).unwrap().len() && at.y < len - 1 {
            let next_row = self.rows.remove(at.y + 1);
            let row = self.rows.get_mut(at.y).unwrap();
            row.append(&next_row)
        } else {
            let row = self.rows.get_mut(at.y).unwrap();
            row.delete(at.x)
        };
    }

    /// 保存修改后的文本
    pub fn save(&mut self) -> Result<(), Error> {
        if let Some(file_name) = &self.file_name {
            let mut file = fs::File::create(file_name)?;
            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
            self.dirty = false
        }
        Ok(())
    }

    /// 获取指定行
    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    /// 判断文档是否为空
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
    /// 判断文档是否被修改
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    /// 文档长度(多少行)
    pub fn len(&self) -> usize {
        self.rows.len()
    }
}
