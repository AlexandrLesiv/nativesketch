use crate::{
    buffer::Permit,
    work::{Payload, Stamp},
    *,
};
#[derive(Debug)]
pub struct ProviderWrite {
    pub(crate) stamp: Stamp,
    pub(crate) job: Box<WriteJob>,
    pub(crate) permit: Permit,
    pub(crate) max_bytes: usize,
}
/// Compact move-only handle; buffer, snapshot and reservation share one owned lifetime.
#[derive(Debug)]
pub struct EncodedWrite {
    inner: Box<EncodedState>,
}
#[derive(Debug)]
struct EncodedState {
    stamp: Stamp,
    job: WriteJob,
    _permit: Permit,
    bytes: Vec<u8>,
}
impl ProviderWrite {
    pub fn info(&self) -> WorkInfo {
        self.stamp.info
    }
    pub fn fail(self, failure: StorageFailure) -> WorkResult {
        WorkResult {
            stamp: Box::new(self.stamp),
            payload: Payload::Write(self.job.complete(WriteOutcome::Failed(failure))),
        }
    }
    /// Consume once on a worker; no live editor reference or repeat encoding allocation.
    pub fn encode(mut self) -> Result<EncodedWrite, WorkResult> {
        if !matches!(self.job.encoded_len(), Ok(n) if n <= self.max_bytes) {
            return Err(self.fail(StorageFailure::Capacity));
        }
        match self.job.encode() {
            Ok(bytes) => {
                self.permit.shrink(bytes.len());
                Ok(EncodedWrite {
                    inner: Box::new(EncodedState {
                        stamp: self.stamp,
                        job: *self.job,
                        _permit: self.permit,
                        bytes,
                    }),
                })
            }
            Err(_) => Err(self.fail(StorageFailure::Encode)),
        }
    }
}
impl EncodedWrite {
    pub fn info(&self) -> WorkInfo {
        self.inner.stamp.info
    }
    pub fn bytes(&self) -> &[u8] {
        &self.inner.bytes
    }
    /// Consume only after the provider no longer uses bytes. Invalid provider reports
    /// return this owned object intact so the caller can supply the actual completion.
    pub fn finish(self, outcome: WriteOutcome) -> Result<WorkResult, (AppError, Self)> {
        let access = self
            .inner
            .stamp
            .info
            .resource
            .expect("write resource")
            .access;
        let valid = match outcome {
            WriteOutcome::Written(_) => access.confirmed_write,
            WriteOutcome::DispatchedUnconfirmed => {
                !access.confirmed_write
                    && self.inner.job.token().purpose() == SavePurpose::ExportCopy
            }
            WriteOutcome::Failed(_) | WriteOutcome::Uncertain(_) => true,
        };
        if !valid {
            return Err((AppError::CapabilityDenied, self));
        }
        let EncodedState { stamp, job, .. } = *self.inner;
        Ok(WorkResult {
            stamp: Box::new(stamp),
            payload: Payload::Write(job.complete(outcome)),
        })
    }
}
