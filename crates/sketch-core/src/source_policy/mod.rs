//! Immutable source metadata policy, independent of transport, cache, and renderer.
//!
//! The host authenticates the namespace, verifies the content hash against bytes,
//! validates the document, and correlates selection/request identity before applying
//! this decision. Only identical immutable metadata is a duplicate. Accept advances
//! metadata even when bytes match, preserving ordering and ancestry. No clock orders revisions.
mod comparison;
mod version;
pub use comparison::{Decision, compare};
pub use version::{Kind, Version};
#[cfg(test)]
mod tests;
