//! Versioned, bounded transport for the portable owner. No host or I/O policy.
#![forbid(unsafe_code)]
mod decode;
mod decode_commands;
mod decode_records;
mod error;
mod failure;
mod reader;
mod request;
mod tags;
mod write_tags;
pub use decode::decode;
pub use error::WireError;
pub use failure::Failure;
pub use request::{Packet, RawContext, Request};
use sketch_app::*;
use sketch_core::application::{routing::*, storage::*, *};
mod budget;
mod identity;
mod limits;
mod owner;
mod reply;
pub use limits::TransportLimits;
pub use owner::WireApp;
pub use reply::Reply;
mod diagnostics;
mod dispatch;
mod exchange;
mod projection;
mod session_queries;
mod work;
mod worker_call;
pub use work::{InputBytes, WireResult, WireWork, WorkerReply};
mod scene_budget;
mod scene_query;
mod scene_records;
mod scene_store;
#[cfg(test)]
mod tests;
