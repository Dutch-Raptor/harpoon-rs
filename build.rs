#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon_with_id("src/assets/harpoon.ico", "1");
    res.compile().unwrap();
}

#[cfg(unix)]
fn main() {}
