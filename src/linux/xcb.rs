use bitflags::bitflags;
use library_loader::Linker;
use log::unwrap_f;
use std::mem::zeroed;
use std::os::raw::{c_char, c_int, c_uint, c_void};

bitflags! {
    pub(crate) struct EventMask: u32 {
        const NO_EVENT = 0;
        const KEY_PRESS = 1;
        const KEY_RELEASE = 2;
        const BUTTON_PRESS = 4;
        const BUTTON_RELEASE = 8;
        const ENTER_WINDOW = 16;
        const LEAVE_WINDOW = 32;
        const POINTER_MOTION = 64;
        const POINTER_MOTION_HINT = 128;
        const BUTTON_1_MOTION = 256;
        const BUTTON_2_MOTION = 512;
        const BUTTON_3_MOTION = 1024;
        const BUTTON_4_MOTION = 2048;
        const BUTTON_5_MOTION = 4096;
        const BUTTON_MOTION = 8192;
        const KEYMAP_STATE = 16384;
        const EXPOSURE = 32768;
        const VISIBILITY_CHANGE = 65536;
        const STRUCTURE_NOTIFY = 131072;
        const RESIZE_REDIRECT = 262144;
        const SUBSTRUCTURE_NOTIFY = 524288;
        const SUBSTRUCTURE_REDIRECT = 1048576;
        const FOCUS_CHANGE = 2097152;
        const PROPERTY_CHANGE = 4194304;
        const COLOR_MAP_CHANGE = 8388608;
        const OWNER_GRAB_BUTTON = 16777216;
    }
}

bitflags! {
    pub(crate) struct CW: u32 {
        const BACK_PIXMAP = 1;
        const BACK_PIXEL = 2;
        const BORDER_PIXMAP = 4;
        const BORDER_PIXEL = 8;
        const BIT_GRAVITY = 16;
        const WIN_GRAVITY = 32;
        const BACKING_STORE = 64;
        const BACKING_PLANES = 128;
        const BACKING_PIXEL = 256;
        const OVERRIDE_REDIRECT = 512;
        const SAVE_UNDER = 1024;
        const EVENT_MASK = 2048;
        const DONT_PROPAGATE = 4096;
        const COLORMAP = 8192;
        const CURSOR = 16384;
    }
}

#[repr(u32)]
#[derive(Copy, Clone)]
#[cfg_attr(debug_mode, derive(Debug))]
pub(crate) enum WindowClass {
    _CopyFromParent = 0,
    InputOutput = 1,
    _InputOnly = 2,
}

#[repr(u32)]
#[derive(Copy, Clone)]
#[cfg_attr(debug_mode, derive(Debug))]
pub(crate) enum AtomEnum {
    _None = 0,
    _Primary = 1,
    _Secondary = 2,
    _Arc = 3,
    _Atom = 4,
    _Bitmap = 5,
    _Cardinal = 6,
    _Colormap = 7,
    _Cursor = 8,
    _CutBuffer0 = 9,
    _CutBuffer1 = 10,
    _CutBuffer2 = 11,
    _CutBuffer3 = 12,
    _CutBuffer4 = 13,
    _CutBuffer5 = 14,
    _CutBuffer6 = 15,
    _CutBuffer7 = 16,
    _Drawable = 17,
    _Font = 18,
    _Integer = 19,
    _Pixmap = 20,
    _Point = 21,
    _Rectangle = 22,
    _ResourceManager = 23,
    _RgbColorMap = 24,
    _RgbBestMap = 25,
    _RgbBlueMap = 26,
    _RgbDefaultMap = 27,
    _RgbGrayMap = 28,
    _RgbGreenMap = 29,
    _RgbRedMap = 30,
    String = 31,
    _VisualId = 32,
    _Window = 33,
    _WmCommand = 34,
    _WmHints = 35,
    _WmClientMachine = 36,
    _WmIconName = 37,
    _WmIconSize = 38,
    WmName = 39,
    _WmNormalHints = 40,
    _WmSizeHints = 41,
    _WmZoomHints = 42,
    _MinSpace = 43,
    _NormSpace = 44,
    _MaxSpace = 45,
    _EndSpace = 46,
    _SuperscriptX = 47,
    _SuperscriptY = 48,
    _SubscriptX = 49,
    _SubscriptY = 50,
    _UnderlinePosition = 51,
    _UnderlineThickness = 52,
    _StrikeoutAscent = 53,
    _StrikeoutDescent = 54,
    _ItalicAngle = 55,
    _XHeight = 56,
    _QuadWidth = 57,
    _Weight = 58,
    _PointSize = 59,
    _Resolution = 60,
    _Copyright = 61,
    _Notice = 62,
    _FontName = 63,
    _FamilyName = 64,
    _FullName = 65,
    _CapHeight = 66,
    _WmClass = 67,
    _WmTransientFor = 68,
}

#[repr(u32)]
pub(crate) enum ButtonIndex {
    _IndexAny = 0,
    Index1 = 1,
    Index2 = 2,
    Index3 = 3,
    _Index4 = 4,
    _Index5 = 5,
}

#[derive(Copy, Clone)]
#[cfg_attr(debug_mode, derive(Debug))]
pub enum Connection {}

pub type Window = u32;
pub(crate) type ColorMap = u32;
pub(crate) type VisualId = u32;
pub(crate) type Atom = u32;
pub(crate) type KeyCode = u8;
pub(crate) type Button = u8;
pub(crate) type TimeStamp = u32;
pub(crate) type ColormapAlloc = u32;
pub(crate) type ButtonReleaseEvent = ButtonPressEvent;
pub(crate) type KeyReleaseEvent = KeyPressEvent;

pub const COLORMAP_ALLOC_NONE: ColormapAlloc = 0;

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg_attr(debug_mode, derive(Debug))]
pub struct Screen {
    pub(crate) root: Window,
    pub(crate) default_colormap: ColorMap,
    pub(crate) white_pixel: u32,
    pub(crate) black_pixel: u32,
    pub(crate) current_input_masks: u32,
    pub(crate) width_in_pixels: u16,
    pub(crate) height_in_pixels: u16,
    pub(crate) width_in_millimeters: u16,
    pub(crate) height_in_millimeters: u16,
    pub(crate) min_installed_maps: u16,
    pub(crate) max_installed_maps: u16,
    pub(crate) root_visual: VisualId,
    pub(crate) backing_stores: u8,
    pub(crate) save_unders: u8,
    pub(crate) root_depth: u8,
    pub(crate) allowed_depths_len: u8,
}

impl Default for Screen {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg_attr(debug_mode, derive(Debug))]
pub(crate) struct InternAtomReply {
    pub(crate) response_type: u8,
    pub(crate) pad0: u8,
    pub(crate) sequence: u16,
    pub(crate) length: u32,
    pub(crate) atom: Atom,
}

impl Default for InternAtomReply {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg_attr(debug_mode, derive(Debug))]
pub(crate) struct Setup {
    pub(crate) status: u8,
    pub(crate) pad0: u8,
    pub(crate) protocol_major_version: u16,
    pub(crate) protocol_minor_version: u16,
    pub(crate) length: u16,
    pub(crate) release_number: u32,
    pub(crate) resource_id_base: u32,
    pub(crate) resource_id_mask: u32,
    pub(crate) motion_buffer_size: u32,
    pub(crate) vendor_len: u16,
    pub(crate) maximum_request_length: u16,
    pub(crate) roots_len: u8,
    pub(crate) pixmap_formats_len: u8,
    pub(crate) image_byte_order: u8,
    pub(crate) bitmap_format_bit_order: u8,
    pub(crate) bitmap_format_scanline_unit: u8,
    pub(crate) bitmap_format_scanline_pad: u8,
    pub(crate) min_keycode: KeyCode,
    pub(crate) max_keycode: KeyCode,
    pub(crate) pad1: [u8; 4usize],
}

impl Default for Setup {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg_attr(debug_mode, derive(Debug))]
pub(crate) struct ScreenIterator {
    pub(crate) data: *mut Screen,
    pub(crate) rem: c_int,
    pub(crate) index: c_int,
}

impl Default for ScreenIterator {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg_attr(debug_mode, derive(Debug))]
pub(crate) struct VoidCookie {
    pub(crate) sequence: c_uint,
}

impl Default for VoidCookie {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg_attr(debug_mode, derive(Debug))]
pub(crate) struct InternAtomCookie {
    pub(crate) sequence: c_uint,
}

impl Default for InternAtomCookie {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg_attr(debug_mode, derive(Debug))]
pub(crate) struct GenericError {
    pub(crate) response_type: u8,
    pub(crate) error_code: u8,
    pub(crate) sequence: u16,
    pub(crate) resource_id: u32,
    pub(crate) minor_code: u16,
    pub(crate) major_code: u8,
    pub(crate) pad0: u8,
    pub(crate) pad: [u32; 5usize],
    pub(crate) full_sequence: u32,
}

impl Default for GenericError {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg_attr(debug_mode, derive(Debug))]
pub(crate) struct GenericEvent {
    pub(crate) response_type: u8,
    pub(crate) pad0: u8,
    pub(crate) sequence: u16,
    pub(crate) pad: [u32; 7usize],
    pub(crate) full_sequence: u32,
}

impl Default for GenericEvent {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg_attr(debug_mode, derive(Debug))]
pub(crate) struct ClientMessageEvent {
    pub(crate) response_type: u8,
    pub(crate) format: u8,
    pub(crate) sequence: u16,
    pub(crate) window: Window,
    pub(crate) type_: Atom,
    pub(crate) data: ClientMessageData,
}

impl Default for ClientMessageEvent {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg_attr(debug_mode, derive(Debug))]
pub(crate) struct ClientMessageData {
    pub(crate) data: [u32; 5usize],
}

impl Default for ClientMessageData {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

#[repr(C)]
pub(crate) struct ButtonPressEvent {
    pub(crate) response_type: u8,
    pub(crate) detail: Button,
    pub(crate) sequence: u16,
    pub(crate) time: TimeStamp,
    pub(crate) root: Window,
    pub(crate) event: Window,
    pub(crate) child: Window,
    pub(crate) root_x: i16,
    pub(crate) root_y: i16,
    pub(crate) event_x: i16,
    pub(crate) event_y: i16,
    pub(crate) state: u16,
    pub(crate) same_screen: u8,
    pub(crate) pad0: u8,
}

#[repr(C)]
pub(crate) struct KeyPressEvent {
    pub(crate) response_type: u8,
    pub(crate) detail: KeyCode,
    pub(crate) sequence: u16,
    pub(crate) time: TimeStamp,
    pub(crate) root: Window,
    pub(crate) event: Window,
    pub(crate) child: Window,
    pub(crate) root_x: i16,
    pub(crate) root_y: i16,
    pub(crate) event_x: i16,
    pub(crate) event_y: i16,
    pub(crate) state: u16,
    pub(crate) same_screen: u8,
    pub(crate) pad0: u8,
}

#[repr(C)]
pub(crate) struct ConfigureNotifyEvent {
    pub(crate) response_type: u8,
    pub(crate) pad0: u8,
    pub(crate) sequence: u16,
    pub(crate) event: Window,
    pub(crate) window: Window,
    pub(crate) above_sibling: Window,
    pub(crate) x: i16,
    pub(crate) y: i16,
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub(crate) border_width: u16,
    pub(crate) override_redirect: u8,
    pub(crate) pad1: u8,
}

#[repr(C)]
pub(crate) struct ResizeRequestEvent {
    pub(crate) response_type: u8,
    pub(crate) pad0: u8,
    pub(crate) sequence: u16,
    pub(crate) window: Window,
    pub(crate) width: u16,
    pub(crate) height: u16,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg_attr(debug_mode, derive(Debug))]
pub(crate) struct QueryPointerCookie {
    pub(crate) sequence: c_uint,
}

impl Default for QueryPointerCookie {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg_attr(debug_mode, derive(Debug))]
pub(crate) struct QueryPointerReply {
    pub(crate) response_type: u8,
    pub(crate) same_screen: u8,
    pub(crate) sequence: u16,
    pub(crate) length: u32,
    pub(crate) root: Window,
    pub(crate) child: Window,
    pub(crate) root_x: i16,
    pub(crate) root_y: i16,
    pub(crate) win_x: i16,
    pub(crate) win_y: i16,
    pub(crate) mask: u16,
    pub(crate) pad0: [u8; 2usize],
}

impl Default for QueryPointerReply {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

pub(crate) const COPY_FROM_PARENT: u64 = 0;

pub(crate) struct Xcb {
    // pub(crate) connect: unsafe extern "C" fn(displayname: *const c_char, screenp: *mut c_int) -> *mut Connection,
    pub(crate) get_setup: extern "C" fn(c: *mut Connection) -> *const Setup,
    pub(crate) setup_roots_iterator: extern "C" fn(R: *const Setup) -> ScreenIterator,
    pub(crate) screen_next: extern "C" fn(i: *mut ScreenIterator),
    pub(crate) generate_id: extern "C" fn(c: *mut Connection) -> u32,
    pub(crate) create_window: extern "C" fn(
        c: *mut Connection,
        depth: u8,
        wid: Window,
        parent: Window,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        border_width: u16,
        class: u16,
        visual: VisualId,
        value_mask: u32,
        value_list: *const u32,
    ) -> VoidCookie,
    pub(crate) intern_atom: extern "C" fn(
        c: *mut Connection,
        only_if_exists: u8,
        name_len: u16,
        name: *const c_char,
    ) -> InternAtomCookie,
    pub(crate) intern_atom_reply: extern "C" fn(
        c: *mut Connection,
        cookie: InternAtomCookie,
        e: *mut *mut GenericError,
    ) -> *mut InternAtomReply,
    pub(crate) change_property: extern "C" fn(
        c: *mut Connection,
        mode: u8,
        window: Window,
        property: Atom,
        type_: Atom,
        format: u8,
        data_len: u32,
        data: *const c_void,
    ) -> VoidCookie,
    pub(crate) map_window: extern "C" fn(c: *mut Connection, window: Window) -> VoidCookie,
    pub(crate) flush: extern "C" fn(c: *mut Connection) -> c_int,
    pub(crate) poll_for_event: extern "C" fn(c: *mut Connection) -> *mut GenericEvent,
    pub(crate) query_pointer:
        extern "C" fn(c: *mut Connection, window: Window) -> QueryPointerCookie,
    pub(crate) query_pointer_reply: extern "C" fn(
        c: *mut Connection,
        cookie: QueryPointerCookie,
        e: *mut *mut GenericError,
    ) -> *mut QueryPointerReply,
    pub(crate) destroy_window: extern "C" fn(*mut Connection, Window) -> VoidCookie,
    // pub(crate) disconnect: extern "C" fn(*mut Connection),
    pub(crate) create_colormap:
        extern "C" fn(*mut Connection, u8, ColorMap, Window, VisualId) -> VoidCookie,
    _lib: Linker,
}

impl Xcb {
    pub(crate) fn new() -> Self {
        let _lib = unwrap_f!(Linker::new("libxcb.so"));
        macro_rules! fun {
            ($f:ident) => {
                unwrap_f!(_lib.get_function(&concat!("xcb_", stringify!($f))))
            };
        }
        Self {
            // connect: fun!(connect),
            get_setup: fun!(get_setup),
            setup_roots_iterator: fun!(setup_roots_iterator),
            screen_next: fun!(screen_next),
            generate_id: fun!(generate_id),
            create_window: fun!(create_window),
            intern_atom: fun!(intern_atom),
            intern_atom_reply: fun!(intern_atom_reply),
            change_property: fun!(change_property),
            map_window: fun!(map_window),
            flush: fun!(flush),
            poll_for_event: fun!(poll_for_event),
            query_pointer: fun!(query_pointer),
            query_pointer_reply: fun!(query_pointer_reply),
            destroy_window: fun!(destroy_window),
            // disconnect: fun!(disconnect),
            create_colormap: fun!(create_colormap),
            _lib,
        }
    }
}

#[cfg(feature = "debug_derive")]
impl std::fmt::Debug for Xcb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Xcb-Library")
    }
}
