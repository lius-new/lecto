use std::fs;

use super::row::Row;

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
}

impl Document {
    /// 打开一个文档
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(filename)?;
        let mut rows = Vec::new();

        for value in contents.lines() {
            rows.push(Row::from(value))
        }

        Ok(Self { rows })
    }

    /// 获取指定行
    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    /// 判断文档是否为空
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}
