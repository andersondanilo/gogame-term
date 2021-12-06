mod core;
mod gogame;

use crate::gogame::GoGame;
use iced_tui::Application;

fn main() {
    GoGame::run();
}
