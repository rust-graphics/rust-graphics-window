#[repr(C)]
pub(crate) struct ANativeActivity {
    pub callbacks: &'static mut ANativeActivityCallbacks,
    pub vm: *mut JavaVM,
    pub env: *mut JNIEnv,
    pub class: jobject,
    pub internalDataPath: *const c_char,
    pub externalDataPath: *const c_char,
    pub sdkVersion: i32,
    pub instance: *mut c_void,
    pub assetManager: *mut AAssetManager,
    pub obbPath: *const c_char,
}
