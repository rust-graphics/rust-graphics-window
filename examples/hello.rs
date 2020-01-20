extern crate rust_graphics_window as window;
use std::{thread, time};

fn main() {
    let w = window::Window::new();
    thread::sleep(time::Duration::from_secs(2));
}
