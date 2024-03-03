/// mod define
mod document;
mod editors;
mod row;
mod terminal;

/// mod using
pub use editors::Editor;
pub use editors::Position;
pub use row::Row;
pub use terminal::Terminal;

/// application running
fn main() {
    Editor::default().run();
}
