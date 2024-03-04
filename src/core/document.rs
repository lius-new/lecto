use std::fs;

use super::{editor::Position, row::Row};

#[derive(Default, Debug)]
pub struct Document {
    rows: Vec<Row>,
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
        })
    }

    /// 插入字符
    pub fn inesrt(&mut self, at: &Position, c: char) {
        if at.y == self.len() {
            let mut row = Row::default();
            row.insert(0, c);
            self.rows.push(row)
        } else if at.y < self.len() {
            let row = self.rows.get_mut(at.y).unwrap();
            row.insert(at.x, c);
        }
    }

    /// 获取指定行
    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    /// 判断文档是否为空
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// 文档长度(多少行)
    pub fn len(&self) -> usize {
        self.rows.len()
    }
}
