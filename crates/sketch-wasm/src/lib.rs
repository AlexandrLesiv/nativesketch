//! Dependency-free WebAssembly adapter. Protocol and lifetimes: ../WASM.md.
mod buffers;
mod command;
mod dispatch;
mod encoding;
mod handles;
mod reader;
mod registry;
pub use buffers::*;
pub use command::sketch_wasm_command;
pub use handles::*;
#[cfg(test)]
mod tests;
