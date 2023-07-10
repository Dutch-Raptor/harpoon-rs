use harpoon::Harpoon;

mod config;
mod harpoon;
mod keyboard;
mod quick_menu;
mod window;

fn main() {
    let mut harpoon = Harpoon::new();
    harpoon.run();
}
