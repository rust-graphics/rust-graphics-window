use {
    super::{super::event::Engine as EventEngine, glue::AndroidApp},
    std::{
        mem::transmute,
        os::raw::{c_char, c_void},
        ptr::null_mut,
        sync::{Arc, RwLock},
    },
};

pub struct Window {
    android_app: &'static mut AndroidApp,
    event_engine: EventEngine,
}

#[cfg(feature = "debug_derive")]
impl std::fmt::Debug for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Android Window")
    }
}

impl Window {
    pub fn new(android_app: *mut c_void) -> Self {
        Self {
            android_app: unsafe { transmute(android_app) },
            event_engine: EventEngine::new(),
        }
    }

    // pub fn initialize(&self) {
    //     let mut events = 0 as c_int;
    //     let mut source = 0 as *mut AndroidPollSource;
    //     while unsafe { (*self.and_app).destroy_requested == 0 } {
    //         if unsafe { ALooper_pollAll(-1, null_mut(), &mut events, transmute(&mut source)) } >= 0
    //         {
    //             if source != null_mut() {
    //                 unsafe {
    //                     ((*source).process)(self.and_app, source);
    //                 }
    //             }
    //             if unsafe { (*self.and_app).window != null_mut() } {
    //                 return;
    //             }
    //         }
    //     }
    //     log_e!("Unexpected flow.");
    // }

    // pub fn set_renderer(&mut self, renderer: Arc<RwLock<RenderEngine>>) {
    //     self.renderer = Some(renderer);
    // }

    // pub fn run(&self) {
    //     loop {
    //         let _ = self.fetch_events();
    //         vxresult!(vxunwrap!(&self.renderer).read()).update();
    //     }
    // }

    // fn handle_cmd(&self, cmd: i32) {
    //     match unsafe { transmute::<i8, AppCmd>(cmd as i8) } {
    //         AppCmd::InitWindow => {
    //             log_i!("Window has been shown!");
    //         }
    //         AppCmd::TermWindow => {
    //             log_i!("Window has been terminated!");
    //         }
    //         c @ _ => {
    //             let _ = c;
    //             log_i!("event {:?} not handled.", c);
    //         }
    //     }
    // }

    // fn handle_input(&self, e: &input::AInputEvent) -> i32 {
    //     let et = unsafe { input::AInputEvent_getType(e) };
    //     if et & input::AInputEventType::Motion as i32 != 0 {
    //         let ea = unsafe { input::AMotionEvent_getAction(e) };
    //         let a: input::AMotionEventAction = unsafe { transmute(ea & 0xFF) };
    //         let pi = (ea & 0xFF00) >> 8;
    //         let fi = unsafe { input::AMotionEvent_getPointerId(e, pi as usize) };
    //         let ww = unsafe { window::ANativeWindow_getWidth((*self.and_app).window) } as Real;
    //         let wh = unsafe { window::ANativeWindow_getHeight((*self.and_app).window) } as Real;
    //         match a {
    //             input::AMotionEventAction::PointerDown | input::AMotionEventAction::Down => {
    //                 let e = Event::new(EventType::Touch(Touch::Raw {
    //                     index: fi as FingerIndexType,
    //                     action: TouchAction::Press,
    //                     point: (
    //                         unsafe { input::AMotionEvent_getRawX(e, pi as usize) } / ww,
    //                         unsafe { input::AMotionEvent_getRawY(e, pi as usize) } / wh,
    //                     ),
    //                 }));
    //                 let ge = vxresult!(self.gesture_translator.write()).receive(&e);
    //                 let core_app = vxresult!(vxunwrap!(&self.core_app).read());
    //                 core_app.on_event(e);
    //                 for e in ge {
    //                     core_app.on_event(e);
    //                 }
    //                 return 1;
    //             }
    //             input::AMotionEventAction::PointerUp | input::AMotionEventAction::Up => {
    //                 let e = Event::new(EventType::Touch(Touch::Raw {
    //                     index: fi as FingerIndexType,
    //                     action: TouchAction::Release,
    //                     point: (
    //                         unsafe { input::AMotionEvent_getX(e, pi as usize) } / ww,
    //                         unsafe { input::AMotionEvent_getY(e, pi as usize) } / wh,
    //                     ),
    //                 }));
    //                 let ge = vxresult!(self.gesture_translator.write()).receive(&e);
    //                 let core_app = vxresult!(vxunwrap!(&self.core_app).read());
    //                 core_app.on_event(e);
    //                 for e in ge {
    //                     core_app.on_event(e);
    //                 }
    //                 return 1;
    //             }
    //             input::AMotionEventAction::Move => {
    //                 let hs = unsafe { input::AMotionEvent_getHistorySize(e) };
    //                 let current = (
    //                     unsafe { input::AMotionEvent_getRawX(e, pi as usize) } / ww,
    //                     unsafe { input::AMotionEvent_getRawY(e, pi as usize) } / wh,
    //                 );
    //                 let previous = if hs > 0 {
    //                     (
    //                         unsafe {
    //                             input::AMotionEvent_getHistoricalRawX(e, pi as usize, hs - 1)
    //                         } / ww,
    //                         unsafe {
    //                             input::AMotionEvent_getHistoricalRawY(e, pi as usize, hs - 1)
    //                         } / wh,
    //                     )
    //                 } else {
    //                     current
    //                 };
    //                 let e = Event::new(EventType::Move(Move::Touch {
    //                     index: fi as FingerIndexType,
    //                     previous,
    //                     current,
    //                     delta: (current.0 - previous.0, current.1 - previous.1),
    //                 }));
    //                 let ge = vxresult!(self.gesture_translator.write()).receive(&e);
    //                 let core_app = vxresult!(vxunwrap!(&self.core_app).read());
    //                 core_app.on_event(e);
    //                 for e in ge {
    //                     core_app.on_event(e);
    //                 }
    //                 return 1;
    //             }
    //             _ => (),
    //         }
    //     } else if et & input::AInputEventType::Key as i32 != 0 {
    //         vxunimplemented!();
    //     } else {
    //         vxunexpected!();
    //     }

    //     0
    // }

    // pub fn fetch_events(&self) -> Vec<Event> {
    //     let mut events = 0 as c_int;
    //     let mut source = 0 as *mut AndroidPollSource;
    //     while unsafe {
    //         (*self.and_app).destroy_requested == 0
    //             && ALooper_pollAll(0, null_mut(), &mut events, transmute(&mut source)) >= 0
    //     } && source != null_mut()
    //     {
    //         unsafe {
    //             ((*source).process)(self.and_app, source);
    //         }
    //     }
    //     let events = vxresult!(self.events.read()).clone();
    //     vxresult!(self.events.write()).clear();
    //     return events;
    // }

    // pub fn get_window_aspect_ratio(&self) -> f32 {
    //     1.7
    // }
}

// extern "C" fn handle_cmd(android_app: *mut AndroidApp, cmd: i32) {
//     unsafe {
//         vxresult!(vxunwrap!(&(*android_app).os_app).read()).handle_cmd(cmd);
//     }
// }

// extern "C" fn handle_input(android_app: *mut AndroidApp, event: *mut input::AInputEvent) -> i32 {
//     unsafe {
//         return vxresult!(vxunwrap!(&(*android_app).os_app).read()).handle_input(transmute(event));
//     }
// }

// impl Drop for Window {
//     fn drop(&mut self) {
//         log_e!(
//             "Error unexpected deletion of Os Window this is a \
//              TODO I will decide later how to do finall termination."
//         );
//     }
// }

// use super::rect::ARect;

// #[repr(u32)]
// pub enum WindowFormat {
//     Rgba8888 = 1,
//     Rgbx8888 = 2,
//     Rgb565 = 4,
// }

pub type ANativeWindow = c_void;

#[repr(C)]
pub struct ANativeWindowBuffer {
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub format: i32,
    pub bits: *mut c_void,
    pub reserved: [u32; 6usize],
}

// impl Default for ANativeWindowBuffer {
//     fn default() -> Self {
//         unsafe { zeroed() }
//     }
// }

// #[cfg_attr(target_os = "android", link(name = "android", kind = "dylib"))]
// extern "C" {
//     pub fn ANativeWindow_acquire(window: *mut ANativeWindow);
//     pub fn ANativeWindow_release(window: *mut ANativeWindow);
//     pub fn ANativeWindow_getWidth(window: *mut ANativeWindow) -> i32;
//     pub fn ANativeWindow_getHeight(window: *mut ANativeWindow) -> i32;
//     pub fn ANativeWindow_getFormat(window: *mut ANativeWindow) -> i32;
//     pub fn ANativeWindow_setBuffersGeometry(
//         window: *mut ANativeWindow,
//         width: i32,
//         height: i32,
//         format: i32,
//     ) -> i32;
//     pub fn ANativeWindow_lock(
//         window: *mut ANativeWindow,
//         out_buffer: *mut ANativeWindowBuffer,
//         in_out_dirty_bounds: *mut ARect,
//     ) -> i32;
//     pub fn ANativeWindow_unlockAndPost(window: *mut ANativeWindow) -> i32;
// }

#[macro_export]
macro_rules! create_window {
    () => {
        crate::window::Window::new(android_app)
    };
}
