use {
    super::{asset::AAssetManager, input::AInputQueue, jni, rect::ARect, window::ANativeWindow},
    std::os::raw::{c_char, c_int, c_void},
};

#[repr(C)]
pub struct ANativeActivity {
    pub callbacks: &'static mut ANativeActivityCallbacks,
    pub vm: &'static mut jni::JavaVM,
    pub env: &'static mut jni::JNIEnv,
    pub class: jni::JObject,
    pub internal_data_path: *const c_char,
    pub external_data_path: *const c_char,
    pub sdk_version: i32,
    pub instance: *mut c_void,
    pub asset_manager: *mut AAssetManager,
    pub obb_path: *const c_char,
}

pub type ActivityReceiverFNP = extern "C" fn(activity: *mut ANativeActivity);
pub type ActivitySizeReceiverFNP =
    extern "C" fn(activity: *mut ANativeActivity, size: *mut usize) -> *mut c_void;
pub type ActivityIntReceiverFNP = extern "C" fn(activity: *mut ANativeActivity, hasFocus: c_int);
pub type ActivityWindowReceiverFNP =
    extern "C" fn(activity: *mut ANativeActivity, window: *mut ANativeWindow);
pub type ActivityInputReceiverFNP =
    extern "C" fn(activity: *mut ANativeActivity, queue: *mut AInputQueue);
pub type ActivityRectReceiverFNP =
    extern "C" fn(activity: *mut ANativeActivity, rect: *const ARect);

#[repr(C)]
pub struct ANativeActivityCallbacks {
    pub on_start: ActivityReceiverFNP,
    pub on_resume: ActivityReceiverFNP,
    pub on_save_instance_state: ActivitySizeReceiverFNP,
    pub on_pause: ActivityReceiverFNP,
    pub on_stop: ActivityReceiverFNP,
    pub on_destroy: ActivityReceiverFNP,
    pub on_window_focus_changed: ActivityIntReceiverFNP,
    pub on_native_window_created: ActivityWindowReceiverFNP,
    pub on_native_window_resized: ActivityWindowReceiverFNP,
    pub on_native_window_redraw_needed: ActivityWindowReceiverFNP,
    pub on_native_window_destroyed: ActivityWindowReceiverFNP,
    pub on_input_queue_created: ActivityInputReceiverFNP,
    pub on_input_queue_destroyed: ActivityInputReceiverFNP,
    pub on_content_rect_changed: ActivityRectReceiverFNP,
    pub on_configuration_changed: ActivityReceiverFNP,
    pub on_low_memory: ActivityReceiverFNP,
}

#[cfg_attr(target_os = "android", link(name = "android", kind = "dylib"))]
extern "C" {
    // pub fn ANativeActivity_finish(activity: *mut ANativeActivity);
    // pub fn ANativeActivity_setWindowFormat(activity: *mut ANativeActivity, format: i32);
    // pub fn ANativeActivity_setWindowFlags(
    //     activity: *mut ANativeActivity,
    //     addFlags: u32,
    //     removeFlags: u32,
    // );
    // pub fn ANativeActivity_showSoftInput(activity: *mut ANativeActivity, flags: u32);
    // pub fn ANativeActivity_hideSoftInput(activity: *mut ANativeActivity, flags: u32);
}

// #[repr(u32)]
// pub enum ShowSoftInputFlagBits {
//     IMPLICIT = 0x0001,
//     FORCED = 0x0002,
// }

// #[repr(u32)]
// pub enum HideSoftInputFlagBits {
//     IMPLICIT_ONLY = 0x0001,
//     NOT_ALWAYS = 0x0002,
// }
