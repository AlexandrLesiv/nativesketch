mod codec;
mod file_store;
mod reader;
#[cfg(test)]
mod tests;
pub(crate) use codec::{decode, encode};
pub(crate) use file_store::{load, save};
pub(crate) const MAX_BYTES: u64 = 64 * 1024 * 1024;
pub(crate) mod snapshot;
