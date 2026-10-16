//! One native mutation owner, bounded effects, and an ordered worker. No native widgets.
mod config;
pub use config::RuntimeConfig;

mod io;
mod io_roots;
mod io_run;
mod io_state;
pub mod work;
pub use io::NativeIo;
mod io_diagnostics;
mod io_perform;
mod io_prepare;
mod owner;
mod owner_types;
pub use owner::NativeRuntime;
pub use owner_types::{
    ActionResult, EventKind, IntentEffect, NativeEvent, RecoveryInventory, RuntimeError,
    RuntimeState, SessionSlot,
};
mod apply;
mod checkpoints;
mod commands;
mod effects;
mod grants;
mod lifecycle;
mod locations;
mod poll;
mod receipt;
mod recovery_owner;
mod recovery_results;
mod release;

mod retirement;

#[cfg(feature = "native-test-controls")]
mod test_control;
#[cfg(feature = "native-test-controls")]
pub use test_control::TestControl;

mod io_events;

mod operation;

mod recovery_retry;

mod private_root;

#[cfg(test)]
mod poll_tests;
