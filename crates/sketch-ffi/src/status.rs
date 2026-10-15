use sketch_core::Error;
use std::panic::{AssertUnwindSafe, catch_unwind};
pub(crate) fn status(error: Error) -> u32 {
    match error {
        Error::InvalidArgument => 1,
        Error::InvalidState => 2,
        Error::Io => 4,
        Error::Format => 5,
    }
}
pub(crate) fn protect(f: impl FnOnce() -> Result<(), u32>) -> u32 {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(())) => 0,
        Ok(Err(e)) => e,
        Err(_) => 6,
    }
}
