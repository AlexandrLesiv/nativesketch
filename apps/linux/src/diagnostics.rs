use std::{
    fmt::Arguments,
    sync::atomic::{AtomicUsize, Ordering},
};
static REMAINING: AtomicUsize = AtomicUsize::new(128);
/// Bounded process diagnostic budget; callers pass typed IDs/errors, never private paths.
pub(crate) fn record(message: Arguments<'_>) {
    if REMAINING
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_sub(1))
        .is_ok()
    {
        eprintln!("{message}");
    }
}
