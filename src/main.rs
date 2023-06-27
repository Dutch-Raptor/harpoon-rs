use harpoon::Harpoon;

mod harpoon;
mod quick_menu;
mod window;

#[macro_use]
extern crate lazy_static;

fn main() {
    let mut harpoon = Harpoon::new();
    harpoon.run();
}
