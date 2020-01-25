#![feature(drain_filter)]
extern crate bitflags;
extern crate rust_graphics_library_loader as liblod;
extern crate rust_graphics_log as log;

pub mod event;

#[cfg(target_os = "android")]
pub mod android;
#[cfg(target_os = "android")]
pub use android::window::*;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux::window::*;

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::window::*;
