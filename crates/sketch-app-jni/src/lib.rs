//! JNI transport v1 over the safe portable application bridge. No host policy.
//!
//! Each JVM caller owns private direct packet/receipt buffers, with exclusive access
//! for the whole native call. Inputs are immutable while borrowed. JNI local refs
//! and environments never cross threads; opaque bridge handles retain their rules.
//! See docs/architecture/022-android-jni-contract.md for publication and cleanup.
#![deny(unsafe_op_in_unsafe_fn)]
mod app;
mod boundary;
mod buffers;
mod direct;
mod invocation;
mod receipt;
mod work;
use sketch_app_bridge as bridge;
// Reexport the existing JNI entry for the isolated pending-exception test library.
pub use app::Java_dev_nativesketch_portable_PortableJniV1_appCall;

#[cfg(test)]
mod tests;
