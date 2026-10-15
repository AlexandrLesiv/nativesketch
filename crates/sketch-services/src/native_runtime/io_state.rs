use super::{
    RuntimeConfig,
    io_roots::{self, RecoveryAccess},
    work::*,
};
use crate::{diagnostics::DiagnosticRing, storage::DestinationAccess};
use sketch_core::application::storage::DestinationId;
use std::{path::PathBuf, time::Instant};
pub(super) struct Destination {
    pub id: DestinationId,
    pub path: PathBuf,
    pub access: DestinationAccess,
    pub leases: usize,
}
pub(super) struct IoState {
    pub config: RuntimeConfig,
    pub recovery: Result<RecoveryAccess, NativeFailure>,
    pub diagnostics: Result<io_roots::DiagnosticAccess, NativeFailure>,
    pub ring: DiagnosticRing,
    pub destinations: Vec<Destination>,
    pub next_destination: u64,
    pub started: Instant,
}
impl IoState {
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            recovery: io_roots::recovery(&config),
            diagnostics: io_roots::diagnostics(&config),
            ring: DiagnosticRing::new(256, 20).expect("fixed valid limits"),
            destinations: Vec::new(),
            next_destination: 1,
            started: Instant::now(),
            config,
        }
    }
    pub fn resolve(&mut self, path: PathBuf) -> Result<DestinationId, NativeFailure> {
        DestinationAccess::new(path.clone()).map_err(|_| NativeFailure::Access)?;
        let canonical = if path.exists() {
            path.canonicalize()
        } else {
            path.parent()
                .ok_or(NativeFailure::Access)?
                .canonicalize()
                .map(|parent| parent.join(path.file_name().expect("validated filename")))
        }
        .map_err(|_| NativeFailure::Access)?;
        if let Some(entry) = self
            .destinations
            .iter_mut()
            .find(|entry| entry.path == canonical)
        {
            entry.leases = entry.leases.checked_add(1).ok_or(NativeFailure::Capacity)?;
            return Ok(entry.id);
        }
        if self.destinations.len() >= self.config.sessions * 2 + self.config.operations {
            return Err(NativeFailure::Capacity);
        }
        let access =
            DestinationAccess::new(canonical.clone()).map_err(|_| NativeFailure::Access)?;
        let id = DestinationId(self.next_destination);
        self.next_destination = self
            .next_destination
            .checked_add(1)
            .ok_or(NativeFailure::Capacity)?;
        self.destinations.push(Destination {
            id,
            path: canonical,
            access,
            leases: 1,
        });
        Ok(id)
    }
    pub fn release(&mut self, id: DestinationId) {
        if let Some(index) = self.destinations.iter().position(|entry| entry.id == id) {
            self.destinations[index].leases -= 1;
            if self.destinations[index].leases == 0 {
                self.destinations.remove(index);
            }
        }
    }
    pub fn result(&mut self, operation: u64, value: WorkerValue) -> WorkerResult {
        if let Ok(sink) = &mut self.diagnostics {
            self.ring.drain_to(|event| sink.sink.write(event));
        }
        let stats = self.ring.stats();
        WorkerResult {
            operation,
            value,
            diagnostic_drops: stats.dropped + stats.rate_limited,
            diagnostic_sink_failures: stats.sink_failures + u64::from(self.diagnostics.is_err()),
        }
    }
}
