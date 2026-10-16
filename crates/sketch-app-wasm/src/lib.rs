//! Synchronous Wasm-v2 handles; no platform policy or legacy ABI replacement.
#![cfg(target_arch = "wasm32")]
#![deny(unsafe_op_in_unsafe_fn)]
mod app;
mod buffers;
mod record;
mod work;
use record::{Output, run};
use sketch_app_bridge as bridge;
