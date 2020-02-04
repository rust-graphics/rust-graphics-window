pub extern crate js_sys;
pub extern crate wasm_bindgen;
pub extern crate web_sys;

use {
    self::wasm_bindgen::{prelude::*, JsCast},
    super::super::event::Engine,
    log::{log_i, result_f, unwrap_f},
    std::sync::Arc,
};

pub struct Window {
    window: web_sys::Window,
    document: web_sys::Document,
    canvas: web_sys::HtmlCanvasElement,
    event_engine: Engine,
}

impl Window {
    pub fn new(_: ()) -> Arc<Self> {
        let window = unwrap_f!(web_sys::window());
        let document = unwrap_f!(window.document());
        let canvas = unwrap_f!(document.get_element_by_id("canvas"));
        let event_engine = Engine::new();
        {
            let e: &web_sys::HtmlElement = unwrap_f!(canvas.dyn_ref());
            event_engine.init_window_aspects(e.offset_width() as i64, e.offset_height() as i64);
            event_engine.init_mouse_position((0, 0));
        }
        let canvas = result_f!(canvas.dyn_into());
        let result = Arc::new(Self {
            window,
            document,
            canvas,
            event_engine,
        });
        macro_rules! set_event {
            ($f1:ident, $f2:ident) => {{
                let win = result.clone();
                let f = Closure::wrap(Box::new(move || win.$f1()) as Box<dyn FnMut()>);
                result.window.$f2(Some(f.as_ref().unchecked_ref()));
                f.forget()
            }};
        }
        set_event!(resized, set_onresize);
        set_event!(init_mouse, set_onmousemove);
        set_event!(init_mouse, set_onmouseenter);
        result
    }

    fn resized(&self) {
        let e: &web_sys::HtmlElement = unwrap_f!(self.canvas.dyn_ref());
        self.event_engine
            .window_size_changed(e.offset_width() as i64, e.offset_height() as i64);
    }

    fn init_mouse(&self) {
        log_i!("Init mouse")
    }
}
