use std::io::{self, stdout};

use termion::{event::Key, input::TermRead, raw::IntoRawMode};

fn die(error: std::io::Error) {
    panic!("{:?}", error)
}

fn main() -> Result<(), Box<io::Error>> {
    let _stdout = stdout().into_raw_mode()?;

    for key in io::stdin().keys() {
        match key {
            Ok(key) => match key {
                Key::Ctrl('q') => break,
                Key::Ctrl(c) => {
                    println!("control: {:?}\r", c);
                }
                Key::Char(c) => {
                    println!("char {:?}\r", c);
                }
                _ => (),
            },
            Err(err) => die(err),
        }
    }

    Ok(())
}
