use crate::{
    buffer::Permit,
    work::{Payload, Stamp},
    *,
};
#[derive(Debug)]
pub struct ProviderRead {
    pub(crate) stamp: Stamp,
    pub(crate) ticket: ReplacementTicket,
    pub(crate) _permit: Permit,
    pub(crate) max_bytes: usize,
}
impl ProviderRead {
    pub fn info(&self) -> WorkInfo {
        self.stamp.info
    }
    pub fn maximum_bytes(&self) -> usize {
        self.max_bytes
    }
    pub fn fail(self, failure: StorageFailure) -> WorkResult {
        WorkResult {
            stamp: Box::new(self.stamp),
            payload: Payload::Failure(Some(self.ticket.fail(failure))),
        }
    }
    /// Host limits reads to maximum_bytes()+1, then passes the owned bounded buffer here.
    /// Decode and disposal occur on this worker; no partial candidate enters live state.
    pub fn prepare(self, bytes: Vec<u8>) -> WorkResult {
        if bytes.len() > self.max_bytes {
            return self.fail(StorageFailure::Capacity);
        }
        match ValidatedDocument::decode(&bytes) {
            Ok(document) => WorkResult {
                stamp: Box::new(self.stamp),
                payload: prepared(self.ticket.prepare(Some(document))),
            },
            Err(_) => self.fail(StorageFailure::Encode),
        }
    }
}
#[derive(Debug)]
pub struct ProviderNew {
    pub(crate) stamp: Stamp,
    pub(crate) ticket: ReplacementTicket,
}
impl ProviderNew {
    pub fn prepare(self) -> WorkResult {
        WorkResult {
            stamp: Box::new(self.stamp),
            payload: prepared(self.ticket.prepare(None)),
        }
    }
}

fn prepared(result: Result<PreparedReplacement, ReplacementFailure>) -> Payload {
    match result {
        Ok(value) => Payload::Candidate(Box::new(value)),
        Err(value) => Payload::Failure(Some(value)),
    }
}
