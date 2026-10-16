//! Portable owner and issued provider work. No I/O, native widgets, renderer or executor.
//! Keep the app on one mutation owner. Move WorkItem to a worker; return WorkResult.
//! An abandoned active write deliberately keeps ordering/shutdown busy: dropping work
//! is not evidence that provider I/O terminated. Hosts must finish/fail issued work.
mod apply_ready;
mod buffer;
mod cancellation;
mod capture;
mod completion;
mod diagnostics;
mod dispatch;
pub use diagnostics::{AppDiagnostic, DiagnosticOutcome};
mod effects;
mod ids;
mod limits;
mod operations;
mod owner;
mod read_work;
mod resource_supply;
mod resources;
mod session;
mod take_work;
mod types;
mod work;
mod write_work;
pub use effects::{AppOutcome, HostEffect};
pub use ids::{DestinationHandle, OperationId, SessionHandle};
pub use limits::AppLimits;
pub use owner::PortableApp;
pub use read_work::{ProviderNew, ProviderRead};
use sketch_core::application::{routing::*, storage::*, *};
pub use types::*;
pub use work::{RetiredState, WorkInfo, WorkItem, WorkKind, WorkResult};
pub use write_work::{EncodedWrite, ProviderWrite};
#[cfg(test)]
mod tests;

pub use sketch_core::application::routing::SaveDestinationPolicy;
