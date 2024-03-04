use termion::color;

/// constants

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239);
pub const STATUS_FG_COLOR: color::Rgb = color::Rgb(63, 63, 63);
