use {
    super::{glx, x11, x11_xcb, xcb, xproto},
    crate::event::*,
    log::{log_e, log_f, log_i},
    std::{
        ffi::CString,
        mem::{transmute, transmute_copy},
        os::raw::{c_int, c_uint},
        ptr::{null, null_mut},
        sync::Arc,
    },
};

pub struct Window {
    x11_lib: x11::X11,
    display: *mut x11::Display,
    _x11_xcb_lib: x11_xcb::X11Xcb,
    glx_lib: glx::Glx,
    #[cfg(feature = "gl")]
    glx_window: glx::Window,
    #[cfg(feature = "gl")]
    glx_context: glx::Context,
    xcb_lib: xcb::Xcb,
    connection: *mut xcb::Connection,
    screen: &'static mut xcb::Screen,
    window: xcb::Window,
    atom_wm_delete_window: *mut xcb::InternAtomReply,
    event_engine: Engine,
}

impl Window {
    pub fn new(_: ()) -> Arc<Self> {
        let x11_lib = x11::X11::new();
        let xcb_lib = xcb::Xcb::new();
        let _x11_xcb_lib = x11_xcb::X11Xcb::new();
        let glx_lib = glx::Glx::new();

        let display = (x11_lib.open_display)(null());
        if display == null_mut() {
            log_f!("Can not open X11 display.");
        }
        let default_screen = (x11_lib.default_screen)(display);

        let connection = (_x11_xcb_lib.get_xcb_connection)(display);
        if connection.is_null() {
            log_f!("Could not find a XCB connection.");
        }
        (_x11_xcb_lib.set_event_queue_owner)(display, x11_xcb::XCB_OWNS_EVENT_QUEUE);
        let setup = (xcb_lib.get_setup)(connection);
        let mut iter = (xcb_lib.setup_roots_iterator)(setup);
        for _ in 0..default_screen {
            if iter.rem == 0 {
                break;
            }
            (xcb_lib.screen_next)(&mut iter);
        }
        let screen: &'static mut xcb::Screen = unsafe { transmute(iter.data) };

        let mut value_list = [0u32; 3];
        value_list[0] = screen.black_pixel;
        value_list[1] = (xcb::EventMask::KEY_RELEASE
            | xcb::EventMask::KEY_PRESS
            | xcb::EventMask::EXPOSURE
            | xcb::EventMask::STRUCTURE_NOTIFY
            | xcb::EventMask::POINTER_MOTION
            | xcb::EventMask::BUTTON_PRESS
            | xcb::EventMask::BUTTON_RELEASE
            | xcb::EventMask::RESIZE_REDIRECT)
            .bits();
        #[cfg(feature = "vulkan")]
        let value_mask = (xcb::CW::BACK_PIXEL | xcb::CW::EVENT_MASK).bits();
        let window: xcb::Window = (xcb_lib.generate_id)(connection);
        let window_width = 1000;
        let window_height = 500;

        #[cfg(feature = "vulkan")]
        (xcb_lib.create_window)(
            connection,
            xcb::COPY_FROM_PARENT as u8,
            window,
            screen.root,
            0,
            0,
            window_width,
            window_height,
            0,
            xcb::WindowClass::InputOutput as u16,
            screen.root_visual,
            value_mask,
            value_list.as_ptr(),
        );

        #[cfg(feature = "gl")]
        let (glx_context, glx_window) = {
            let visual_attribs = [
                glx::X_RENDERABLE,
                glx::TRUE,
                glx::DRAWABLE_TYPE,
                glx::WINDOW_BIT,
                glx::RENDER_TYPE,
                glx::RGBA_BIT,
                glx::X_VISUAL_TYPE,
                glx::TRUE_COLOR,
                glx::RED_SIZE,
                8,
                glx::GREEN_SIZE,
                8,
                glx::BLUE_SIZE,
                8,
                glx::ALPHA_SIZE,
                8,
                glx::DEPTH_SIZE,
                24,
                glx::STENCIL_SIZE,
                8,
                glx::DOUBLEBUFFER,
                glx::TRUE,
                glx::SAMPLE_BUFFERS,
                1,
                glx::SAMPLES,
                4,
                glx::NONE,
            ];

            let mut visual_id: c_int = 0;
            let mut num_fb_configs: c_int = 0;
            let fb_configs = (glx_lib.choose_fb_config)(
                display,
                default_screen,
                visual_attribs.as_ptr(),
                &mut num_fb_configs,
            );
            if fb_configs.is_null() || num_fb_configs == 0 {
                log_f!("glXGetFBConfigs failed");
            }

            #[cfg(feature = "verbose-log")]
            log_i!("Found {} matching FB configs", num_fb_configs);

            let fb_config = unsafe { *fb_configs };
            if glx::SUCCESS
                != (glx_lib.get_fb_config_attrib)(
                    display,
                    fb_config,
                    glx::VISUAL_ID,
                    &mut visual_id,
                )
            {
                log_f!("Failed to get Visual ID");
            }

            #[cfg(feature = "verbose-log")]
            log_i!(
                "Visual ID of the selected FB config is {} and Root Visual ID is {}.",
                visual_id,
                screen.root_visual
            );

            let context = (glx_lib.create_new_context)(
                display,
                fb_config,
                glx::RGBA_TYPE,
                null_mut(),
                glx::TRUE,
            );
            if context.is_null() {
                log_f!("glXCreateNewContext failed");
            }

            let colormap = (xcb_lib.generate_id)(connection);
            (xcb_lib.create_colormap)(
                connection,
                xcb::COLORMAP_ALLOC_NONE as u8,
                colormap,
                screen.root,
                visual_id as xcb::VisualId,
            );
            value_list[0] = value_list[1];
            value_list[1] = colormap;
            let value_mask = (xcb::CW::COLORMAP | xcb::CW::EVENT_MASK).bits();
            (xcb_lib.create_window)(
                connection,
                xcb::COPY_FROM_PARENT as u8,
                window,
                screen.root,
                0,
                0,
                window_width,
                window_height,
                0,
                xcb::WindowClass::InputOutput as u16,
                visual_id as xcb::VisualId,
                value_mask,
                value_list.as_ptr(),
            );
            (xcb_lib.map_window)(connection, window);
            let glx_window = (glx_lib.create_window)(display, fb_config, window as glx::Window, 0);
            if glx_window == 0 {
                log_f!("Failed to generate GLX Window.");
            }
            if 0 == (glx_lib.make_context_current)(display, glx_window, glx_window, context) {
                log_f!("Can not make the glx context current.");
            }
            (context, glx_window)
        };

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
            x11_lib,
            display,
            _x11_xcb_lib,
            glx_lib,
            glx_window,
            glx_context,
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
        Arc::new(result)
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
            xproto::KEY_CAPS_LOCK_0 => Keyboard::CapsLock(0),
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
                pad: false,
            },
            xproto::KEY_NUM_1 => Keyboard::Number {
                number: 1,
                pad: false,
            },
            xproto::KEY_NUM_2 => Keyboard::Number {
                number: 2,
                pad: false,
            },
            xproto::KEY_NUM_3 => Keyboard::Number {
                number: 3,
                pad: false,
            },
            xproto::KEY_NUM_4 => Keyboard::Number {
                number: 4,
                pad: false,
            },
            xproto::KEY_NUM_5 => Keyboard::Number {
                number: 5,
                pad: false,
            },
            xproto::KEY_NUM_6 => Keyboard::Number {
                number: 6,
                pad: false,
            },
            xproto::KEY_NUM_7 => Keyboard::Number {
                number: 7,
                pad: false,
            },
            xproto::KEY_NUM_8 => Keyboard::Number {
                number: 8,
                pad: false,
            },
            xproto::KEY_NUM_9 => Keyboard::Number {
                number: 9,
                pad: false,
            },
            xproto::KEY_NUM_PAD_0 => Keyboard::Number {
                number: 0,
                pad: true,
            },
            xproto::KEY_NUM_PAD_1 => Keyboard::Number {
                number: 1,
                pad: true,
            },
            xproto::KEY_NUM_PAD_2 => Keyboard::Number {
                number: 2,
                pad: true,
            },
            xproto::KEY_NUM_PAD_3 => Keyboard::Number {
                number: 3,
                pad: true,
            },
            xproto::KEY_NUM_PAD_4 => Keyboard::Number {
                number: 4,
                pad: true,
            },
            xproto::KEY_NUM_PAD_5 => Keyboard::Number {
                number: 5,
                pad: true,
            },
            xproto::KEY_NUM_PAD_6 => Keyboard::Number {
                number: 6,
                pad: true,
            },
            xproto::KEY_NUM_PAD_7 => Keyboard::Number {
                number: 7,
                pad: true,
            },
            xproto::KEY_NUM_PAD_8 => Keyboard::Number {
                number: 8,
                pad: true,
            },
            xproto::KEY_NUM_PAD_9 => Keyboard::Number {
                number: 9,
                pad: true,
            },
            k @ _ => {
                log_e!("Unknown key: {:?} pressed", k);
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
        unsafe { transmute_copy(&self.screen) }
    }

    fn get_mouse_position(&self) -> (i64, i64) {
        let cookie = (self.xcb_lib.query_pointer)(self.connection, self.window);
        let replay = (self.xcb_lib.query_pointer_reply)(self.connection, cookie, null_mut());
        if replay.is_null() {
            log_f!("Can not fetch mouse position.");
        }
        let reply: &mut xcb::QueryPointerReply = unsafe { transmute(replay) };
        let result = (reply.root_x as i64, reply.root_y as i64);
        unsafe {
            libc::free(transmute(reply));
        }
        result
    }

    pub fn get_event_engine(&self) -> &Engine {
        &self.event_engine
    }

    #[cfg(feature = "gl")]
    pub fn swap(&self) {
        (self.glx_lib.swap_buffers)(self.display, self.glx_window);
    }

    #[cfg(feature = "gl")]
    pub fn get_gl_function<T>(&self, s: &str) -> Option<T> {
        let cs = CString::new(s).unwrap();
        if let Some(f) = (self.glx_lib.get_proc_address)(cs.as_ptr()) {
            Some(unsafe { transmute_copy(&f) })
        } else {
            None
        }
    }
}

unsafe impl Send for Window {}

unsafe impl Sync for Window {}

#[cfg(feature = "debug-derive")]
impl std::fmt::Debug for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Xcb-Window")
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        #[cfg(feature = "gl")]
        (self.glx_lib.destroy_window)(self.display, self.glx_window);
        (self.xcb_lib.destroy_window)(self.connection, self.window);
        #[cfg(feature = "gl")]
        (self.glx_lib.destroy_context)(self.display, self.glx_context);
        (self.x11_lib.close_display)(self.display);
        #[cfg(feature = "verbose-log")]
        log_i!("Rust-Graphics Window dropped.");
    }
}
