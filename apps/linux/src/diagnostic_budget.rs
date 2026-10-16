use std::time::Duration;
#[derive(Clone, Copy)]
pub(crate) enum Level {
    Failure = 0,
    Event = 1,
}
pub(crate) struct Budget {
    start: Duration,
    used: [u32; 2],
    suppressed: [u64; 2],
}
impl Budget {
    pub fn new() -> Self {
        Self {
            start: Duration::ZERO,
            used: [0; 2],
            suppressed: [0; 2],
        }
    }
    pub fn admit(&mut self, now: Duration, level: Level) -> bool {
        if now.saturating_sub(self.start) >= Duration::from_secs(1) {
            self.start = now;
            self.used = [0; 2];
        }
        let index = level as usize;
        if self.used[index] < 16 {
            self.used[index] += 1;
            true
        } else {
            self.suppressed[index] = self.suppressed[index].saturating_add(1);
            false
        }
    }
    pub fn suppressed(&self) -> [u64; 2] {
        self.suppressed
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn successful_events_cannot_spend_failure_budget_and_time_restores_both() {
        let mut budget = Budget::new();
        for _ in 0..16 {
            assert!(budget.admit(Duration::ZERO, Level::Event));
        }
        for _ in 0..1000 {
            assert!(!budget.admit(Duration::ZERO, Level::Event));
        }
        assert!(budget.admit(Duration::ZERO, Level::Failure));
        for _ in 1..16 {
            assert!(budget.admit(Duration::ZERO, Level::Failure));
        }
        assert!(!budget.admit(Duration::from_millis(999), Level::Failure));
        assert_eq!(budget.suppressed(), [1, 1000]);
        assert!(budget.admit(Duration::from_secs(1), Level::Failure));
        assert!(budget.admit(Duration::from_secs(1), Level::Event));
        assert_eq!(budget.suppressed(), [1, 1000]);
        assert!(budget.admit(Duration::from_secs(3600), Level::Failure));
    }
}
