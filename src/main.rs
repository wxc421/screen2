use glium::{implement_vertex, Texture2d};
use imgui::*;
use screenshots::Screen;

mod support;

mod util;



fn main() {
    let screens = Screen::all().unwrap();
    println!("capturer {screens:?}");
    support::run();
}