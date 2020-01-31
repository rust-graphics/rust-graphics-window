use {
    super::x11,
    library_loader::Linker,
    log::unwrap_f,
    std::os::raw::{c_int, c_ulong, c_void},
};

pub const TRUE: c_int = 1;
// pub const FALSE: c_int = 0;
pub const X_RENDERABLE: c_int = 0x8012;
pub const DRAWABLE_TYPE: c_int = 0x8010;
pub const WINDOW_BIT: c_int = 1;
pub const RENDER_TYPE: c_int = 0x8011;
pub const RGBA_BIT: c_int = 1;
pub const X_VISUAL_TYPE: c_int = 34;
pub const TRUE_COLOR: c_int = 32770;
pub const RED_SIZE: c_int = 8;
pub const GREEN_SIZE: c_int = 9;
pub const BLUE_SIZE: c_int = 10;
pub const ALPHA_SIZE: c_int = 11;
pub const DEPTH_SIZE: c_int = 12;
pub const STENCIL_SIZE: c_int = 13;
pub const DOUBLEBUFFER: c_int = 5;
pub const SAMPLE_BUFFERS: c_int = 100000;
pub const SAMPLES: c_int = 100001;
pub const NONE: c_int = 0;
pub const VISUAL_ID: c_int = 32779;
pub const RGBA_TYPE: c_int = 32788;
pub const SUCCESS: c_int = 0;

pub type FBConfig = *mut c_void;
pub type Context = *mut c_void;
pub type XID = c_ulong;
pub type Window = XID;
pub type Drawable = XID;

pub struct Glx {
    pub choose_fb_config:
        extern "C" fn(*mut x11::Display, c_int, *const c_int, *mut c_int) -> *mut FBConfig,
    pub get_fb_config_attrib:
        extern "C" fn(*mut x11::Display, FBConfig, c_int, *mut c_int) -> c_int,
    pub create_new_context:
        extern "C" fn(*mut x11::Display, FBConfig, c_int, Context, c_int) -> Context,
    pub create_window: extern "C" fn(*mut x11::Display, FBConfig, Window, c_int) -> Window,
    pub make_context_current:
        extern "C" fn(*mut x11::Display, Drawable, Drawable, Context) -> c_int,
    pub swap_buffers: extern "C" fn(*mut x11::Display, Window),
    pub get_proc_address: extern "C" fn(*const i8) -> Option<extern "C" fn()>,
    pub destroy_window: extern "C" fn(*mut x11::Display, Window),
    pub destroy_context: extern "C" fn(*mut x11::Display, Context),
    _lib: Linker,
}

impl Glx {
    pub fn new() -> Self {
        let _lib = unwrap_f!(Linker::new("libGLX.so"));
        macro_rules! fun {
            ($f:ident) => {
                unwrap_f!(_lib.get_function(stringify!($f)))
            };
        }
        Self {
            choose_fb_config: fun!(glXChooseFBConfig),
            get_fb_config_attrib: fun!(glXGetFBConfigAttrib),
            create_new_context: fun!(glXCreateNewContext),
            create_window: fun!(glXCreateWindow),
            make_context_current: fun!(glXMakeContextCurrent),
            swap_buffers: fun!(glXSwapBuffers),
            get_proc_address: fun!(glXGetProcAddress),
            destroy_window: fun!(glXDestroyWindow),
            destroy_context: fun!(glXDestroyContext),
            _lib,
        }
    }
}
