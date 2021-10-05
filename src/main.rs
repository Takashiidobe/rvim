#![warn(clippy::all, clippy::pedantic)]
mod editor;
mod terminal;
use self::editor::Editor;
pub use editor::Position;
pub use terminal::Terminal;

fn main() {
    Editor::default().run();
}
