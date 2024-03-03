use std::{
    io::{self, stdout, Read},
    process,
};

use termion::raw::IntoRawMode;

fn main() -> Result<(), Box<io::Error>> {
    let _stdout = stdout().into_raw_mode()?;

    for b in io::stdin().bytes() {
        let b = b.unwrap();
        let c = b as char; // 为什么不是 b.unwrap() as char; 因为unwrap(self)即所有权被函数获取
        println!("{:?}", c);

        // 判断是否是控制字符
        if c.is_control() {
            println!("{:?}\r", b);
        } else {
            println!("{:?}({:?})\r", b, c);
        }

        if c == 'q' {
            process::exit(-1)
        }
    }

    Ok(())
}
