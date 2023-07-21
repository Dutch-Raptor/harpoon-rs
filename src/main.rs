use harpoon::Harpoon;

mod assets;
mod config;
mod harpoon;
mod keyboard;
mod notification;
mod quick_menu;
mod window;

// use anyhow macros
#[macro_use]
extern crate anyhow;

fn main() {
    let mut harpoon = Harpoon::new();
    harpoon.run();
}
