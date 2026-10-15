#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScheduledCheckpoint {
    pub session: u64,
    pub revision: u64,
}
/// Pure owner-side scheduling. Times are monotonic milliseconds, not durability promises.
#[derive(Default)]
pub struct CheckpointSchedule {
    pending: Option<ScheduledCheckpoint>,
    active: Option<ScheduledCheckpoint>,
    first_ms: u64,
    changed_ms: u64,
    suspended: bool,
}
impl CheckpointSchedule {
    /// Call on committed changes only. Predictions/draft updates do not reset scheduling.
    pub fn changed(&mut self, session: u64, revision: u64, now_ms: u64) {
        let request = ScheduledCheckpoint { session, revision };
        if self.pending == Some(request) {
            return;
        }
        if self.pending.is_none() || self.pending.is_some_and(|old| old.session != session) {
            self.first_ms = now_ms;
        }
        self.pending = Some(request);
        self.changed_ms = now_ms;
    }
    /// Explicit saves get priority; active interactions defer copying beyond the 30s target.
    pub fn due(
        &mut self,
        now_ms: u64,
        interaction_active: bool,
        explicit_save_pending: bool,
    ) -> Option<ScheduledCheckpoint> {
        if self.suspended || self.active.is_some() || interaction_active || explicit_save_pending {
            return None;
        }
        if now_ms.saturating_sub(self.changed_ms) < 2000
            && now_ms.saturating_sub(self.first_ms) < 30_000
        {
            return None;
        }
        let request = self.pending.take()?;
        self.active = Some(request);
        Some(request)
    }
    pub fn complete(&mut self, request: ScheduledCheckpoint, capacity_or_failure: bool) -> bool {
        if self.active != Some(request) {
            return false;
        }
        self.active = None;
        if capacity_or_failure {
            self.suspended = true;
            if self.pending.is_none() {
                self.pending = Some(request);
            }
        }
        true
    }
    /// Explicit user/service retry after resolving failure; no unbounded automatic retry.
    pub fn resume(&mut self) {
        self.suspended = false;
    }
    pub fn replace_session(&mut self) {
        self.pending = None;
        self.active = None;
        self.suspended = false;
    }
}
