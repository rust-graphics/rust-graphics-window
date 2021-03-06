extern crate rust_graphics_log as log;
extern crate rust_graphics_main as main;
extern crate rust_graphics_window as window;

use {
    log::{log_i, result_f},
    main::{main, Arg},
    std::sync::{Arc, RwLock},
};

struct Listener {
    pub running: bool,
}

impl window::event::Listener for Listener {
    fn on_event(&mut self, event: &window::event::Event) -> bool {
        match event.get_data() {
            &window::event::Data::Quit => self.running = false,
            _e @ _ => {
                #[cfg(feature = "verbose-log")]
                log_i!("{:?}", _e);
            }
        }
        return false;
    }
}

fn start(arg: Arg) {
    let w = window::Window::new(arg);
    let listener = Arc::new(RwLock::new(Listener { running: true }));
    let l: Arc<RwLock<dyn window::event::Listener>> = listener.clone();
    w.get_event_engine().add(0, Arc::downgrade(&l));
    while { result_f!(listener.read()).running } {
        w.fetch_events();
    }
    log_i!("Program ended.");
}

main!(start);
