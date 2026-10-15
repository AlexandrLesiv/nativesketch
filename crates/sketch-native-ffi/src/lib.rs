//! Independent native ABI v1. Legacy ABI2 remains in sketch-ffi.
#![deny(unsafe_op_in_unsafe_fn)]
mod boundary;
pub mod dto;
pub use boundary::{Runtime, Scene, Session};
#[unsafe(no_mangle)]
pub extern "C" fn sketch_runtime_v1_version() -> u32 {
    1
}
mod availability_api;
mod command_api;
mod convert;
mod effect_api;
mod recovery_api;
mod runtime_api;
mod scene_api;
mod session_api;
#[cfg(feature = "native-test-controls")]
mod test_api;

#[cfg(test)]
mod boundary_tests;
#[cfg(test)]
mod recovery_tests;
#[cfg(target_os = "macos")]
mod renderer_api;
#[cfg(target_os = "macos")]
mod renderer_owner;
#[cfg(all(test, target_os = "macos"))]
mod renderer_tests;
#[cfg(not(target_os = "macos"))]
mod renderer_unsupported;
