use super::super::event::*;
use super::file::AASSET_MANAGER;
use super::glue::{AndroidApp, AndroidPollSource, AppCmd};
use super::input;
use super::looper::ALooper_pollAll;
use super::window;
use std::mem::transmute;
use std::ptr::null_mut;
use std::sync::{Arc, RwLock};

pub struct Window {
    pub and_app: &'static mut AndroidApp,
    event_engine: Engine,
}

#[cfg(feature = "debug_derive")]
impl std::fmt::Debug for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Android Window")
    }
}

impl Window {
    pub fn new(
        activity: *mut activity::ANativeActivity,
        saved_state: *mut libc::c_void,
        saved_state_size: libc::size_t,
    ) -> Self {
        unsafe {
            (*and_app).on_app_cmd = handle_cmd;
            (*and_app).on_input_event = handle_input;
            AASSET_MANAGER = transmute((*(*and_app).activity).assetManager);
        }
        Window {
            core_app: Some(core_app),
            renderer: None,
            and_app,
            events: Arc::new(RwLock::new(Vec::new())),
            gesture_translator: Arc::new(RwLock::new(gesture::Translator::new())),
        }
    }

    pub fn initialize(&self) {
        let mut events = 0 as c_int;
        let mut source = 0 as *mut AndroidPollSource;
        while unsafe { (*self.and_app).destroy_requested == 0 } {
            if unsafe { ALooper_pollAll(-1, null_mut(), &mut events, transmute(&mut source)) } >= 0
            {
                if source != null_mut() {
                    unsafe {
                        ((*source).process)(self.and_app, source);
                    }
                }
                if unsafe { (*self.and_app).window != null_mut() } {
                    return;
                }
            }
        }
        vxloge!("Unexpected flow.");
    }

    pub fn set_renderer(&mut self, renderer: Arc<RwLock<RenderEngine>>) {
        self.renderer = Some(renderer);
    }

    pub fn run(&self) {
        loop {
            let _ = self.fetch_events();
            vxresult!(vxunwrap!(&self.renderer).read()).update();
        }
    }

    fn handle_cmd(&self, cmd: i32) {
        match unsafe { transmute::<i8, AppCmd>(cmd as i8) } {
            AppCmd::InitWindow => {
                vxlogi!("Window has been shown!");
            }
            AppCmd::TermWindow => {
                vxlogi!("Window has been terminated!");
            }
            c @ _ => {
                let _ = c;
                vxlogi!("event {:?} not handled.", c);
            }
        }
    }

    fn handle_input(&self, e: &input::AInputEvent) -> i32 {
        let et = unsafe { input::AInputEvent_getType(e) };
        if et & input::AInputEventType::Motion as i32 != 0 {
            let ea = unsafe { input::AMotionEvent_getAction(e) };
            let a: input::AMotionEventAction = unsafe { transmute(ea & 0xFF) };
            let pi = (ea & 0xFF00) >> 8;
            let fi = unsafe { input::AMotionEvent_getPointerId(e, pi as usize) };
            let ww = unsafe { window::ANativeWindow_getWidth((*self.and_app).window) } as Real;
            let wh = unsafe { window::ANativeWindow_getHeight((*self.and_app).window) } as Real;
            match a {
                input::AMotionEventAction::PointerDown | input::AMotionEventAction::Down => {
                    let e = Event::new(EventType::Touch(Touch::Raw {
                        index: fi as FingerIndexType,
                        action: TouchAction::Press,
                        point: (
                            unsafe { input::AMotionEvent_getRawX(e, pi as usize) } / ww,
                            unsafe { input::AMotionEvent_getRawY(e, pi as usize) } / wh,
                        ),
                    }));
                    let ge = vxresult!(self.gesture_translator.write()).receive(&e);
                    let core_app = vxresult!(vxunwrap!(&self.core_app).read());
                    core_app.on_event(e);
                    for e in ge {
                        core_app.on_event(e);
                    }
                    return 1;
                }
                input::AMotionEventAction::PointerUp | input::AMotionEventAction::Up => {
                    let e = Event::new(EventType::Touch(Touch::Raw {
                        index: fi as FingerIndexType,
                        action: TouchAction::Release,
                        point: (
                            unsafe { input::AMotionEvent_getX(e, pi as usize) } / ww,
                            unsafe { input::AMotionEvent_getY(e, pi as usize) } / wh,
                        ),
                    }));
                    let ge = vxresult!(self.gesture_translator.write()).receive(&e);
                    let core_app = vxresult!(vxunwrap!(&self.core_app).read());
                    core_app.on_event(e);
                    for e in ge {
                        core_app.on_event(e);
                    }
                    return 1;
                }
                input::AMotionEventAction::Move => {
                    let hs = unsafe { input::AMotionEvent_getHistorySize(e) };
                    let current = (
                        unsafe { input::AMotionEvent_getRawX(e, pi as usize) } / ww,
                        unsafe { input::AMotionEvent_getRawY(e, pi as usize) } / wh,
                    );
                    let previous = if hs > 0 {
                        (
                            unsafe {
                                input::AMotionEvent_getHistoricalRawX(e, pi as usize, hs - 1)
                            } / ww,
                            unsafe {
                                input::AMotionEvent_getHistoricalRawY(e, pi as usize, hs - 1)
                            } / wh,
                        )
                    } else {
                        current
                    };
                    let e = Event::new(EventType::Move(Move::Touch {
                        index: fi as FingerIndexType,
                        previous,
                        current,
                        delta: (current.0 - previous.0, current.1 - previous.1),
                    }));
                    let ge = vxresult!(self.gesture_translator.write()).receive(&e);
                    let core_app = vxresult!(vxunwrap!(&self.core_app).read());
                    core_app.on_event(e);
                    for e in ge {
                        core_app.on_event(e);
                    }
                    return 1;
                }
                _ => (),
            }
        } else if et & input::AInputEventType::Key as i32 != 0 {
            vxunimplemented!();
        } else {
            vxunexpected!();
        }

        0
    }

    pub fn fetch_events(&self) -> Vec<Event> {
        let mut events = 0 as c_int;
        let mut source = 0 as *mut AndroidPollSource;
        while unsafe {
            (*self.and_app).destroy_requested == 0
                && ALooper_pollAll(0, null_mut(), &mut events, transmute(&mut source)) >= 0
        } && source != null_mut()
        {
            unsafe {
                ((*source).process)(self.and_app, source);
            }
        }
        let events = vxresult!(self.events.read()).clone();
        vxresult!(self.events.write()).clear();
        return events;
    }

    pub fn get_window_aspect_ratio(&self) -> f32 {
        1.7
    }
}

extern "C" fn handle_cmd(android_app: *mut AndroidApp, cmd: i32) {
    unsafe {
        vxresult!(vxunwrap!(&(*android_app).os_app).read()).handle_cmd(cmd);
    }
}

extern "C" fn handle_input(android_app: *mut AndroidApp, event: *mut input::AInputEvent) -> i32 {
    unsafe {
        return vxresult!(vxunwrap!(&(*android_app).os_app).read()).handle_input(transmute(event));
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        vxloge!(
            "Error unexpected deletion of Os Window this is a \
             TODO I will decide later how to do finall termination."
        );
    }
}
