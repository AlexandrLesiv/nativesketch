//! Disposable rendering only: editor semantics remain in sketch-core.
mod adornments;
mod compositor;
mod paths;
mod request;
mod stats;
mod viewport_path;
pub use compositor::Compositor;
pub use request::{RenderError, RenderRequest};
pub use stats::RenderStats;
#[cfg(target_os = "macos")]
mod metal;
#[cfg(target_os = "macos")]
pub use metal::MetalRenderer;
#[cfg(test)]
mod tests;
