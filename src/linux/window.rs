use super::super::event::*;
use super::xcb;
use super::xproto;

use log::{log_e, log_f, log_i, result_f, unwrap_f};
use std::{
    ffi::CString,
    fmt,
    mem::transmute,
    os::raw::{c_int, c_uint},
    ptr::null_mut,
    sync::{Arc, RwLock},
};

pub struct Window {
    xcb_lib: xcb::Xcb,
    connection: *mut xcb::Connection,
    screen: *mut xcb::Screen,
    window: xcb::Window,
    atom_wm_delete_window: *mut xcb::InternAtomReply,
}

impl Window {
    pub fn new() -> Self {
        let xcb_lib = xcb::Xcb::new();
        let mut scr = 0 as c_int;
        let connection: *mut xcb::Connection = unsafe { (xcb_lib.connect)(null_mut(), &mut scr) };
        if connection == null_mut() {
            log_f!("Could not find a xcb connection.");
        }
        let setup = (xcb_lib.get_setup)(connection);
        let mut iter = (xcb_lib.setup_roots_iterator)(setup);
        for _ in 0..scr {
            (xcb_lib.screen_next)(&mut iter);
        }
        let screen = iter.data;
        let window: xcb::Window = unsafe { transmute((xcb_lib.generate_id)(connection)) };
        let mut value_list = vec![0u32; 32];
        value_list[0] = unsafe { (*screen).black_pixel };
        value_list[1] = (xcb::EventMask::KEY_RELEASE
            | xcb::EventMask::KEY_PRESS
            | xcb::EventMask::EXPOSURE
            | xcb::EventMask::STRUCTURE_NOTIFY
            | xcb::EventMask::POINTER_MOTION
            | xcb::EventMask::BUTTON_PRESS
            | xcb::EventMask::BUTTON_RELEASE
            | xcb::EventMask::RESIZE_REDIRECT)
            .bits();
        let value_mask = (xcb::CW::BACK_PIXEL | xcb::CW::EVENT_MASK).bits();
        let window_width = 1000;
        let window_height = 500;
        (xcb_lib.create_window)(
            connection,
            xcb::COPY_FROM_PARENT as u8,
            window,
            unsafe { (*screen).root },
            0,
            0,
            window_width,
            window_height,
            0,
            xcb::WindowClass::InputOutput as u16,
            unsafe { (*screen).root_visual },
            value_mask,
            value_list.as_ptr(),
        );
        let window_width = window_width as i64;
        let window_height = window_height as i64;
        let window_aspect_ratio = window_width as f64 / window_height as f64;
        /* Magic code that will send notification when window is destroyed */
        let cs = CString::new("WM_PROTOCOLS".to_string().into_bytes()).unwrap();
        let cookie = (xcb_lib.intern_atom)(connection, 1, 12, cs.as_ptr());
        let reply = (xcb_lib.intern_atom_reply)(connection, cookie, null_mut());
        if reply == null_mut() {
            log_f!("Reply is null.");
        }
        let cs = CString::new("WM_DELETE_WINDOW".to_string().into_bytes()).unwrap();
        let cookie2 = (xcb_lib.intern_atom)(connection, 0, 16, cs.as_ptr());
        let atom_wm_delete_window: *mut xcb::InternAtomReply =
            (xcb_lib.intern_atom_reply)(connection, cookie2, null_mut());
        (xcb_lib.change_property)(
            connection,
            xproto::PropMode::Replace as u8,
            window,
            unsafe { (*reply).atom },
            4,
            32,
            1,
            unsafe { transmute(&((*atom_wm_delete_window).atom)) },
        );
        let cs = CString::new("Rust Graphics Window".to_string().into_bytes()).unwrap();
        (xcb_lib.change_property)(
            connection,
            xproto::PropMode::Replace as u8,
            window,
            xcb::AtomEnum::WmName as u32,
            xcb::AtomEnum::String as u32,
            8,
            cs.as_bytes_with_nul().len() as u32,
            unsafe { transmute(cs.as_ptr()) },
        );
        unsafe { libc::free(transmute(reply)) };
        (xcb_lib.map_window)(connection, window);
        (xcb_lib.flush)(connection);
        Self {
            xcb_lib,
            connection,
            screen,
            window,
            atom_wm_delete_window,
        }
    }

    pub fn fetch_events(&mut self) {
        loop {
            let xcb_event = (self.xcb_lib.poll_for_event)(self.connection);
            if xcb_event == null_mut() {
                break;
            }
            self.translate(unsafe { transmute(xcb_event) });
            unsafe {
                libc::free(transmute(xcb_event));
            }
        }
    }

    fn translate(&mut self, e: &xcb::GenericEvent) -> Option<Event> {
        let client_msg: &xcb::ClientMessageEvent = unsafe { transmute(e) };
        match e.response_type as c_uint & 0x7F {
            xproto::DESTROY_NOTIFY => {
                if client_msg.data.data[0] == unsafe { (*self.atom_wm_delete_window).atom } {
                    return Some(Event::new(Data::Quit));
                }
            } // xproto::CLIENT_MESSAGE => {
            //     if client_msg.data.data[0] == unsafe { (*self.atom_wm_delete_window).atom } {}
            // }
            // xproto::MOTION_NOTIFY => {
            //     let pre = (self.mouse_position_x, self.mouse_position_y);
            //     self.update_mouse_position();
            //     let cur = (self.mouse_position_x, self.mouse_position_y);
            //     let delta = (cur.0 - pre.0, cur.1 - pre.1);
            //     if let Some(fun) = self.on_mouse_movment.as_ref() {
            //         fun(pre, cur, delta);
            //     }
            // }
            // xproto::BUTTON_PRESS => {
            //     let press: &mut xcb::ButtonPressEvent = unsafe { transmute(e) };
            //     let m: xcb::ButtonIndex = unsafe { transmute(press.detail as u32) };
            //     let m = match m {
            //         xcb::ButtonIndex::Index1 => Mouse::Left,
            //         xcb::ButtonIndex::Index2 => Mouse::Middle,
            //         xcb::ButtonIndex::Index3 => Mouse::Right,
            //         _ => {
            //             log_i!("Unknown mouse button pressed.");
            //             Mouse::Left
            //         }
            //     };
            //     return Some(EventType::Button {
            //         button: Button::Mouse(m),
            //         action: event::ButtonAction::Press,
            //     });
            // }
            // xproto::BUTTON_RELEASE => {
            //     let release: &mut xcb::ButtonReleaseEvent = unsafe { transmute(e) };
            //     let m: xcb::ButtonIndex = unsafe { transmute(release.detail as u32) };
            //     let m = match m {
            //         xcb::ButtonIndex::_Index1 => Mouse::Left,
            //         xcb::ButtonIndex::_Index2 => Mouse::Middle,
            //         xcb::ButtonIndex::_Index3 => Mouse::Right,
            //         _ => {
            //             log_e!("Unknown mouse button pressed.");
            //             Mouse::Left
            //         }
            //     };
            //     return Some(EventType::Button {
            //         button: Button::Mouse(m),
            //         action: event::ButtonAction::Release,
            //     });
            // }
            // a @ xproto::KEY_PRESS | a @ xproto::KEY_RELEASE => {
            //     let key_event: &xcb::KeyReleaseEvent = unsafe { transmute(e) };
            //     let b = Button::Keyboard(match key_event.detail {
            //         xproto::KEY_W => Keyboard::W,
            //         xproto::KEY_S => Keyboard::S,
            //         xproto::KEY_A => Keyboard::A,
            //         xproto::KEY_D => Keyboard::D,
            //         // xproto::KEY_P => { Keyboard::P },
            //         xproto::KEY_F1 => Keyboard::Function(1),
            //         k @ _ => {
            //             log_i!("Unknown key: {:?} presse", k);
            //             Keyboard::W
            //         }
            //     });
            //     return Some(if a == xproto::KEY_RELEASE {
            //         EventType::Button {
            //             button: b,
            //             action: event::ButtonAction::Release,
            //         }
            //     } else {
            //         EventType::Button {
            //             button: b,
            //             action: event::ButtonAction::Press,
            //         }
            //     });
            // }
            // xproto::CONFIGURE_NOTIFY => {
            //     let cfg_event: &xcb::ConfigureNotifyEvent = unsafe { transmute(e) };
            //     // if cfg_event.width as Real != self.window_aspects.0 ||
            //     //     cfg_event.height as Real != self.window_aspects.1
            //     // {
            //     if cfg_event.width > 0 && cfg_event.height > 0 {
            //         return Some(EventType::Window(Window::SizeChange {
            //             w: cfg_event.width as Real,
            //             h: cfg_event.height as Real,
            //             ratio: (cfg_event.width as Real) / (cfg_event.height as Real),
            //             pre_w: 0.0,
            //             pre_h: 0.0,
            //             pre_ratio: 0.0,
            //         }));
            //     }
            //     // }
            // }
            c @ _ => {
                log_i!("Uncontrolled event: {:?}", c);
            }
        }
        None
    }

    pub(crate) fn get_window(&self) -> xcb::Window {
        return self.window;
    }

    pub(crate) fn get_connection(&self) -> *mut xcb::Connection {
        return self.connection;
    }

    fn get_mouse_position(&self) -> (i64, i64) {
        let reply: &mut xcb::QueryPointerReply = unsafe {
            let coockie = (self.xcb_lib.query_pointer)(self.connection, self.window);
            transmute((self.xcb_lib.query_pointer_reply)(
                self.connection,
                coockie,
                null_mut(),
            ))
        };
        let result = (reply.root_x as i64, reply.root_y as i64);
        unsafe {
            libc::free(transmute(reply));
        }
        result
    }
}

unsafe impl Send for Window {}

unsafe impl Sync for Window {}

impl fmt::Debug for Window {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Xcb-Window")
    }
}
