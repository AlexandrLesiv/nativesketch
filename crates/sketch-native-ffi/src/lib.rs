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
