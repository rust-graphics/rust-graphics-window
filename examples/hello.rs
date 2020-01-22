extern crate rust_graphics_log as log;
extern crate rust_graphics_window as window;

use log::{log_f, log_i, result_f};
use std::sync::{Arc, RwLock};

struct Listener {
    pub running: bool,
}

impl window::event::Listener for Listener {
    fn on_event(&mut self, event: &window::event::Event) -> bool {
        match event.get_data() {
            &window::event::Data::Quit => self.running = false,
            e @ _ => {
                #[cfg_attr(feature = "debug_derive", derive(Debug))]
                log_i!("{:?}", e);
            }
        }
        return false;
    }
}

fn main() {
    let w = window::Window::new();
    let listener = Arc::new(RwLock::new(Listener { running: true }));
    let l: Arc<RwLock<dyn window::event::Listener>> = listener.clone();
    w.get_engine().add(0, Arc::downgrade(&l));
    while { result_f!(listener.read()).running } {
        w.fetch_events();
    }
}
