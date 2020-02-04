use {
    super::{
        super::event::{Engine as EventEngine, FingerIndexType},
        android::{
            glue::{AndroidApp, AndroidPollSource, AppCmd},
            input,
            looper::ALooper_pollAll,
            window,
        },
    },
    log::{log_i, result_f, unexpected_f},
    std::{
        mem::{transmute, transmute_copy},
        os::raw::c_int,
        ptr::null_mut,
        sync::{Arc, Mutex},
    },
};

struct State {
    focused: bool,
    paused: bool,
}

pub struct Window {
    android_app: &'static mut AndroidApp,
    state: Mutex<State>,
    event_engine: EventEngine,
}

#[cfg(feature = "debug-derive")]
impl std::fmt::Debug for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Android Window")
    }
}

impl Window {
    pub fn new(android_app: &'static mut AndroidApp) -> Arc<Self> {
        let result = Arc::new(Self {
            android_app: unsafe { transmute_copy(&android_app) },
            state: Mutex::new(State {
                paused: false,
                focused: false,
            }),
            event_engine: EventEngine::new(),
        });
        android_app.user_data = unsafe { transmute(result.as_ref()) };
        android_app.on_app_cmd = handle_cmd;
        android_app.on_input_event = handle_input;
        result.initialize();
        result
    }

    pub fn get_event_engine(&self) -> &EventEngine {
        &self.event_engine
    }

    pub fn get_window(&self) -> *mut window::ANativeWindow {
        self.android_app.window
    }

    pub fn fetch_events(&self) {
        if self.android_app.destroy_requested != 0 {
            #[cfg(feature = "verbose-log")]
            log_i!("Android app has been terminated already waiting for main loop to terminate.");
            return;
        }
        let timeout: c_int = if result_f!(self.state.lock()).paused {
            10
        } else {
            0
        };
        let mut events = 0 as c_int;
        let mut source = 0 as *mut AndroidPollSource;
        while unsafe { ALooper_pollAll(timeout, null_mut(), &mut events, transmute(&mut source)) }
            >= 0
            && source != null_mut()
        {
            unsafe {
                ((*source).process)(transmute_copy(&self.android_app), source);
            }
        }
    }

    fn initialize(&self) {
        let mut events = 0 as c_int;
        let mut source = 0 as *mut AndroidPollSource;
        while self.android_app.destroy_requested == 0 {
            if unsafe { ALooper_pollAll(-1, null_mut(), &mut events, transmute(&mut source)) } >= 0
            {
                if source != null_mut() {
                    unsafe {
                        ((*source).process)(transmute_copy(&self.android_app), source);
                    }
                }
                if {
                    let state = result_f!(self.state.lock());
                    !state.paused && state.focused
                } {
                    let w =
                        unsafe { window::ANativeWindow_getWidth(self.android_app.window) } as i64;
                    let h =
                        unsafe { window::ANativeWindow_getHeight(self.android_app.window) } as i64;
                    self.event_engine.init_window_aspects(w, h);
                    return;
                }
            }
        }
        unexpected_f!();
    }

    fn handle_cmd(&self, cmd: AppCmd) {
        match cmd {
            AppCmd::InitWindow => {
                #[cfg(feature = "verbose-log")]
                log_i!("Window has been shown!");
            }
            AppCmd::TermWindow => {
                #[cfg(feature = "verbose-log")]
                log_i!("Window has been terminated.");
                self.event_engine.quit();
            }
            AppCmd::GainedFocus => {
                #[cfg(feature = "verbose-log")]
                log_i!("Android app has been focused.");
                result_f!(self.state.lock()).focused = true;
                self.event_engine.window_focus();
            }
            AppCmd::LostFocus => {
                #[cfg(feature = "verbose-log")]
                log_i!("Android app has lost focus.");
                result_f!(self.state.lock()).focused = false;
                self.event_engine.window_defocus();
            }
            AppCmd::Pause => {
                #[cfg(feature = "verbose-log")]
                log_i!("Android app has been paused.");
                result_f!(self.state.lock()).paused = true;
            }
            AppCmd::Start => {
                #[cfg(feature = "verbose-log")]
                log_i!("Android app has been started.");
            }
            AppCmd::Resume => {
                #[cfg(feature = "verbose-log")]
                log_i!("Android app has been resumed.");
                result_f!(self.state.lock()).paused = false;
            }
            AppCmd::SaveState => {
                #[cfg(feature = "verbose-log")]
                log_i!("Android app should save its state.");
            }
            AppCmd::Stop => {
                #[cfg(feature = "verbose-log")]
                log_i!("Android app has been stoped.");
            }
            AppCmd::Destroy => {
                log_i!("Android app has been destroyed.");
            }
            _c @ _ => {
                #[cfg(feature = "verbose-log")]
                log_i!("Event {} not handled.", _c as i32);
            }
        }
    }

    fn handle_input(&self, e: *mut input::AInputEvent) -> i32 {
        let event_type = unsafe { input::AInputEvent_getType(e) };
        if event_type & input::AInputEventType::Motion as i32 != 0 {
            let event_action = unsafe { input::AMotionEvent_getAction(e) };
            let action: input::AMotionEventAction = unsafe { transmute(event_action & 0xFF) };
            let pointer_index = ((event_action & 0xFF00) >> 8) as usize;
            let finger_index =
                unsafe { input::AMotionEvent_getPointerId(e, pointer_index) } as FingerIndexType;
            let pointer_x = unsafe { input::AMotionEvent_getRawX(e, pointer_index) } as i64;
            let pointer_y = unsafe { input::AMotionEvent_getRawY(e, pointer_index) } as i64;
            match action {
                input::AMotionEventAction::PointerDown | input::AMotionEventAction::Down => {
                    self.event_engine
                        .finger_down(pointer_x, pointer_y, finger_index);
                }
                input::AMotionEventAction::PointerUp | input::AMotionEventAction::Up => {
                    self.event_engine
                        .finger_up(pointer_x, pointer_y, finger_index);
                }
                input::AMotionEventAction::Move => {
                    self.event_engine
                        .finger_move(pointer_x, pointer_y, finger_index);
                }
                _ => (),
            }
        } else if event_type & input::AInputEventType::Key as i32 != 0 {
            #[cfg(feature = "verbose-log")]
            log_i!("Unhandled");
        } else {
            unexpected_f!();
        }
        0
    }

    // pub fn get_window_aspect_ratio(&self) -> f32 {
    //     1.7
    // }
}

extern "C" fn handle_cmd(android_app: &mut AndroidApp, cmd: AppCmd) {
    let window: &Window = unsafe { transmute(android_app.user_data) };
    window.handle_cmd(cmd);
}

extern "C" fn handle_input(android_app: &mut AndroidApp, event: *mut input::AInputEvent) -> i32 {
    let window: &Window = unsafe { transmute(android_app.user_data) };
    window.handle_input(event)
}

#[cfg(feature = "verbose-log")]
impl Drop for Window {
    fn drop(&mut self) {
        log_i!("Android Window droped.");
    }
}
