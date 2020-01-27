use super::super::event::*;
use super::xcb;
use super::xproto;

use log::{log_e, log_f, log_i};
use std::{
    ffi::CString,
    mem::transmute,
    os::raw::{c_int, c_uint},
    ptr::null_mut,
};

pub struct Window {
    xcb_lib: xcb::Xcb,
    connection: *mut xcb::Connection,
    screen: *mut xcb::Screen,
    window: xcb::Window,
    atom_wm_delete_window: *mut xcb::InternAtomReply,
    event_engine: Engine,
}

impl Window {
    pub fn new(_: ()) -> Self {
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
        let event_engine = Engine::new();
        event_engine.init_window_aspects(window_width as i64, window_height as i64);
        let result = Self {
            xcb_lib,
            connection,
            screen,
            window,
            atom_wm_delete_window,
            event_engine,
        };
        result
            .event_engine
            .init_mouse_position(result.get_mouse_position());
        result
    }

    pub fn fetch_events(&self) {
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

    fn translate(&self, e: &xcb::GenericEvent) {
        let client_msg: &xcb::ClientMessageEvent = unsafe { transmute(e) };
        match e.response_type as c_uint & 0x7F {
            xproto::DESTROY_NOTIFY => {
                if client_msg.data.data[0] == unsafe { (*self.atom_wm_delete_window).atom } {
                    self.event_engine.quit();
                }
            }
            xproto::CLIENT_MESSAGE => {
                if client_msg.data.data[0] == unsafe { (*self.atom_wm_delete_window).atom } {
                    self.event_engine.quit();
                }
            }
            xproto::MOTION_NOTIFY => {
                self.event_engine
                    .set_mouse_position(self.get_mouse_position());
            }
            xproto::BUTTON_PRESS => {
                let press: &xcb::ButtonPressEvent = unsafe { transmute(e) };
                self.event_engine
                    .button_pressed(Self::translate_mouse_button(press.detail));
            }
            xproto::BUTTON_RELEASE => {
                let release: &xcb::ButtonReleaseEvent = unsafe { transmute(e) };
                self.event_engine
                    .button_released(Self::translate_mouse_button(release.detail));
            }
            xproto::KEY_PRESS => {
                let press: &xcb::KeyPressEvent = unsafe { transmute(e) };
                self.event_engine
                    .button_pressed(Self::translate_key_button(press.detail));
            }
            xproto::KEY_RELEASE => {
                let release: &xcb::KeyReleaseEvent = unsafe { transmute(e) };
                self.event_engine
                    .button_released(Self::translate_key_button(release.detail));
            }
            xproto::CONFIGURE_NOTIFY => {
                let e: &xcb::ConfigureNotifyEvent = unsafe { transmute(e) };
                self.event_engine
                    .window_size_changed(e.width as i64, e.height as i64);
            }
            xproto::RESIZE_REQUEST => {
                let e: &xcb::ResizeRequestEvent = unsafe { transmute(e) };
                self.event_engine
                    .window_size_changed(e.width as i64, e.height as i64);
            }
            c @ _ => {
                log_i!("Uncontrolled event: {:?}", c);
            }
        }
    }

    fn translate_mouse_button(i: u8) -> Button {
        let b: xcb::ButtonIndex = unsafe { transmute(i as u32) };
        Button::Mouse(match b {
            xcb::ButtonIndex::Index1 => Mouse::Left,
            xcb::ButtonIndex::Index2 => Mouse::Middle,
            xcb::ButtonIndex::Index3 => Mouse::Right,
            _ => {
                log_e!("Unexpected mouse button: {}", i);
                Mouse::Unknown(i as u32)
            }
        })
    }

    fn translate_key_button(k: xcb::KeyCode) -> Button {
        Button::Keyboard(match k {
            xproto::KEY_A => Keyboard::A,
            xproto::KEY_B => Keyboard::B,
            xproto::KEY_C => Keyboard::C,
            xproto::KEY_D => Keyboard::D,
            xproto::KEY_E => Keyboard::E,
            xproto::KEY_F => Keyboard::F,
            xproto::KEY_G => Keyboard::G,
            xproto::KEY_H => Keyboard::H,
            xproto::KEY_I => Keyboard::I,
            xproto::KEY_J => Keyboard::J,
            xproto::KEY_K => Keyboard::K,
            xproto::KEY_L => Keyboard::L,
            xproto::KEY_M => Keyboard::M,
            xproto::KEY_N => Keyboard::N,
            xproto::KEY_O => Keyboard::O,
            xproto::KEY_P => Keyboard::P,
            xproto::KEY_Q => Keyboard::Q,
            xproto::KEY_R => Keyboard::R,
            xproto::KEY_S => Keyboard::S,
            xproto::KEY_T => Keyboard::T,
            xproto::KEY_U => Keyboard::U,
            xproto::KEY_V => Keyboard::V,
            xproto::KEY_W => Keyboard::W,
            xproto::KEY_X => Keyboard::X,
            xproto::KEY_Y => Keyboard::Y,
            xproto::KEY_Z => Keyboard::Z,
            xproto::KEY_F1 => Keyboard::Function(1),
            xproto::KEY_F2 => Keyboard::Function(2),
            xproto::KEY_F3 => Keyboard::Function(3),
            xproto::KEY_F4 => Keyboard::Function(4),
            xproto::KEY_F5 => Keyboard::Function(5),
            xproto::KEY_F6 => Keyboard::Function(6),
            xproto::KEY_F7 => Keyboard::Function(7),
            xproto::KEY_F8 => Keyboard::Function(8),
            xproto::KEY_F9 => Keyboard::Function(9),
            xproto::KEY_F10 => Keyboard::Function(10),
            xproto::KEY_F11 => Keyboard::Function(11),
            xproto::KEY_F12 => Keyboard::Function(12),
            xproto::KEY_ENTER_0 => Keyboard::Enter(0),
            xproto::KEY_ENTER_1 => Keyboard::Enter(1),
            xproto::KEY_CTRL_0 => Keyboard::Control(0),
            xproto::KEY_CTRL_1 => Keyboard::Control(1),
            xproto::KEY_ALT_0 => Keyboard::Alt(0),
            xproto::KEY_ALT_1 => Keyboard::Alt(1),
            xproto::KEY_SHIFT_0 => Keyboard::Shift(0),
            xproto::KEY_SHIFT_1 => Keyboard::Shift(1),
            xproto::KEY_ESCAPE_0 => Keyboard::Escape(0),
            xproto::KEY_SPACE_0 => Keyboard::Space(0),
            xproto::KEY_MENU_0 => Keyboard::Menu(0),
            xproto::KEY_DOT_0 => Keyboard::Dot(0),
            xproto::KEY_DOT_1 => Keyboard::Dot(1),
            xproto::KEY_COMMA_0 => Keyboard::Comma(0),
            xproto::KEY_SLASH_0 => Keyboard::Slash(0),
            xproto::KEY_SLASH_1 => Keyboard::Slash(1),
            xproto::KEY_CAPS_LOCK_0 => Keyboard::CapseLock(0),
            xproto::KEY_SEMICOLON => Keyboard::SemiColon,
            xproto::KEY_QUOTE => Keyboard::Quote,
            xproto::KEY_TAB_0 => Keyboard::Tab,
            xproto::KEY_LEFT_BRACKET => Keyboard::BracketLeft,
            xproto::KEY_RIGHT_BRACKET => Keyboard::BracketRight,
            xproto::KEY_BACK_SLASH => Keyboard::BackSlash(0),
            xproto::KEY_BACK_QUOTE => Keyboard::BackQuote,
            xproto::KEY_MINUS_0 => Keyboard::Minus(0),
            xproto::KEY_MINUS_1 => Keyboard::Minus(1),
            xproto::KEY_EQUAL_0 => Keyboard::Equal,
            xproto::KEY_BACKSPACE_0 => Keyboard::Backspace,
            xproto::KEY_PAUSE_BREAK_0 => Keyboard::PauseBreak,
            xproto::KEY_PRINT_SCREEN_0 => Keyboard::PrintScreen,
            xproto::KEY_DELETE_0 => Keyboard::Delete,
            xproto::KEY_HOME_0 => Keyboard::Home,
            xproto::KEY_PAGE_UP_0 => Keyboard::PageUp,
            xproto::KEY_PAGE_DOWN_0 => Keyboard::PageDown,
            xproto::KEY_END_0 => Keyboard::End,
            xproto::KEY_NUMLOCK_0 => Keyboard::NumLock,
            xproto::KEY_STAR_0 => Keyboard::Star,
            xproto::KEY_PLUS_0 => Keyboard::Plus(0),
            xproto::KEY_ARROW_UP => Keyboard::ArrowUp,
            xproto::KEY_ARROW_DOWN => Keyboard::ArrowDown,
            xproto::KEY_ARROW_LEFT => Keyboard::ArrowLeft,
            xproto::KEY_ARROW_RIGHT => Keyboard::ArrowRight,
            xproto::KEY_NUM_0 => Keyboard::Number {
                number: 0,
                padd: false,
            },
            xproto::KEY_NUM_1 => Keyboard::Number {
                number: 1,
                padd: false,
            },
            xproto::KEY_NUM_2 => Keyboard::Number {
                number: 2,
                padd: false,
            },
            xproto::KEY_NUM_3 => Keyboard::Number {
                number: 3,
                padd: false,
            },
            xproto::KEY_NUM_4 => Keyboard::Number {
                number: 4,
                padd: false,
            },
            xproto::KEY_NUM_5 => Keyboard::Number {
                number: 5,
                padd: false,
            },
            xproto::KEY_NUM_6 => Keyboard::Number {
                number: 6,
                padd: false,
            },
            xproto::KEY_NUM_7 => Keyboard::Number {
                number: 7,
                padd: false,
            },
            xproto::KEY_NUM_8 => Keyboard::Number {
                number: 8,
                padd: false,
            },
            xproto::KEY_NUM_9 => Keyboard::Number {
                number: 9,
                padd: false,
            },
            xproto::KEY_NUM_PAD_0 => Keyboard::Number {
                number: 0,
                padd: true,
            },
            xproto::KEY_NUM_PAD_1 => Keyboard::Number {
                number: 1,
                padd: true,
            },
            xproto::KEY_NUM_PAD_2 => Keyboard::Number {
                number: 2,
                padd: true,
            },
            xproto::KEY_NUM_PAD_3 => Keyboard::Number {
                number: 3,
                padd: true,
            },
            xproto::KEY_NUM_PAD_4 => Keyboard::Number {
                number: 4,
                padd: true,
            },
            xproto::KEY_NUM_PAD_5 => Keyboard::Number {
                number: 5,
                padd: true,
            },
            xproto::KEY_NUM_PAD_6 => Keyboard::Number {
                number: 6,
                padd: true,
            },
            xproto::KEY_NUM_PAD_7 => Keyboard::Number {
                number: 7,
                padd: true,
            },
            xproto::KEY_NUM_PAD_8 => Keyboard::Number {
                number: 8,
                padd: true,
            },
            xproto::KEY_NUM_PAD_9 => Keyboard::Number {
                number: 9,
                padd: true,
            },
            k @ _ => {
                log_e!("Unknown key: {:?} presse", k);
                Keyboard::Unknown(k as u32)
            }
        })
    }

    pub fn get_window(&self) -> xcb::Window {
        return self.window;
    }

    pub fn get_connection(&self) -> *mut xcb::Connection {
        return self.connection;
    }

    pub fn get_screen(&self) -> *mut xcb::Screen {
        return self.screen;
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

    pub fn get_engine(&self) -> &Engine {
        &self.event_engine
    }
}

unsafe impl Send for Window {}

unsafe impl Sync for Window {}

#[cfg(feature = "debug_derive")]
impl std::fmt::Debug for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Xcb-Window")
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        (self.xcb_lib.destroy_window)(self.connection, self.window);
        (self.xcb_lib.disconnect)(self.connection);
        #[cfg(feature = "verbose_log")]
        log_i!("Rust-Graphics's Window droped.");
    }
}
