#![feature(drain_filter)]

pub extern crate bitflags;
pub extern crate libc;
pub extern crate rust_graphics_log as log;

#[cfg(not(target_arch = "wasm32"))]
pub extern crate rust_graphics_library_loader as library_loader;

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

#[cfg(target_arch = "wasm32")]
pub mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::window::*;
