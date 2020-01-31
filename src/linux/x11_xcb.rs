use {
    super::{x11, xcb},
    library_loader::Linker,
    log::unwrap_f,
    // std::os::raw::{c_char, c_int, c_void},
};

pub type XEventQueueOwner = u32;

pub const XCB_OWNS_EVENT_QUEUE: XEventQueueOwner = 1;

pub struct X11Xcb {
    pub get_xcb_connection: extern "C" fn(*mut x11::Display) -> *mut xcb::Connection,
    pub set_event_queue_owner: extern "C" fn(*mut x11::Display, XEventQueueOwner),
    _lib: Linker,
}

impl X11Xcb {
    pub fn new() -> Self {
        let _lib = unwrap_f!(Linker::new("libX11-xcb.so"));
        macro_rules! fun {
            ($f:ident) => {
                unwrap_f!(_lib.get_function(stringify!($f)))
            };
        }
        Self {
            get_xcb_connection: fun!(XGetXCBConnection),
            set_event_queue_owner: fun!(XSetEventQueueOwner),
            _lib,
        }
    }
}
