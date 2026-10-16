use crate::WireError;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
pub(crate) fn reserve(count: u64) -> Result<u64, WireError> {
    reserve_from(&NEXT, count)
}
fn reserve_from(counter: &AtomicU64, count: u64) -> Result<u64, WireError> {
    counter
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
            v.checked_add(count)
        })
        .map_err(|_| WireError::capacity())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn global_identity_exhaustion_never_wraps() {
        let counter = AtomicU64::new(u64::MAX - 1);
        assert_eq!(reserve_from(&counter, 1), Ok(u64::MAX - 1));
        assert_eq!(reserve_from(&counter, 1), Err(WireError::capacity()));
        assert_eq!(reserve_from(&counter, 1), Err(WireError::capacity()));
    }
}
