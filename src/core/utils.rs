use std::io::Error;

pub fn die(error: Error) {
    panic!("{:?}", error)
}
