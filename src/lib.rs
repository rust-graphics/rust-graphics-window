extern crate bitflags;
extern crate rust_graphics_library_loader as liblod;
extern crate rust_graphics_log as log;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux::window::*;
