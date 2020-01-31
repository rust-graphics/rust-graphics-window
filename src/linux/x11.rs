use {
    library_loader::Linker,
    log::unwrap_f,
    std::os::raw::{c_char, c_int, c_void},
};

pub type Display = c_void;

pub struct X11 {
    pub open_display: extern "C" fn(*const c_char) -> *mut Display,
    pub close_display: extern "C" fn(*mut Display) -> c_int,
    pub default_screen: extern "C" fn(*mut Display) -> c_int,
    _lib: Linker,
}

impl X11 {
    pub fn new() -> Self {
        let _lib = unwrap_f!(Linker::new("libX11.so"));
        macro_rules! fun {
            ($f:ident) => {
                unwrap_f!(_lib.get_function(stringify!($f)))
            };
        }
        Self {
            open_display: fun!(XOpenDisplay),
            close_display: fun!(XCloseDisplay),
            default_screen: fun!(XDefaultScreen),
            _lib,
        }
    }
}
