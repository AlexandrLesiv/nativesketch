use super::errors::FailureKind;
use std::sync::atomic::{AtomicU64, Ordering};
/// IDs are scoped to this loaded Rust runtime (one Wasm instance on web).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocumentSessionId(u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestId(pub u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Revision(pub u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InteractionId(pub(super) u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneGeneration(pub(super) u64);
static NEXT_SESSION: AtomicU64 = AtomicU64::new(1);
impl DocumentSessionId {
    pub(super) fn fresh() -> Result<Self, FailureKind> {
        reserve(&NEXT_SESSION).map(Self)
    }
    pub fn get(self) -> u64 {
        self.0
    }
}
impl InteractionId {
    pub fn get(self) -> u64 {
        self.0
    }
}
impl SceneGeneration {
    pub fn get(self) -> u64 {
        self.0
    }
}
/// Request identity correlates diagnostics; it grants neither permission nor deduplication.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandContext {
    pub session: DocumentSessionId,
    pub request: RequestId,
    pub interaction: Option<InteractionId>,
}

pub(super) fn reserve(counter: &AtomicU64) -> Result<u64, FailureKind> {
    let mut current = counter.load(Ordering::Relaxed);
    loop {
        let next = current.checked_add(1).ok_or(FailureKind::ResourceLimit)?;
        match counter.compare_exchange_weak(current, next, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => return Ok(current),
            Err(value) => current = value,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exhausted_session_identity_never_wraps_or_reuses() {
        let counter = AtomicU64::new(u64::MAX - 1);
        assert_eq!(reserve(&counter), Ok(u64::MAX - 1));
        assert_eq!(reserve(&counter), Err(FailureKind::ResourceLimit));
        assert_eq!(reserve(&counter), Err(FailureKind::ResourceLimit));
        assert_eq!(counter.load(Ordering::Relaxed), u64::MAX);
    }
}
