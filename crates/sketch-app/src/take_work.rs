use crate::{operations::Phase, work::Stamp, *};
impl PortableApp {
    pub(crate) fn stamp(&self, index: usize, kind: WorkKind) -> Result<Stamp, AppError> {
        let op = &self.operations[index];
        let resource = op
            .resource
            .map(|r| self.resource(r).map(|r| r.descriptor))
            .transpose()?;
        Ok(Stamp {
            app: self.identity,
            info: WorkInfo {
                operation: op.id,
                session: op.session,
                request: op.request,
                resource,
                kind,
            },
        })
    }
    /// Transfer unique work; caller chooses executor. Buffer admission precedes activation.
    pub fn take_work(&mut self) -> Result<Option<WorkItem>, AppError> {
        if let Some(index) = self
            .operations
            .iter()
            .position(|op| matches!(op.phase, Phase::Read(_) | Phase::New(_) | Phase::Retire(_)))
        {
            let (kind, permit) = match self.operations[index].phase {
                Phase::Read(_) => (
                    WorkKind::Read,
                    Some(self.budget.reserve(self.limits.document_bytes)?),
                ),
                Phase::New(_) => (WorkKind::PrepareNew, None),
                _ => (WorkKind::Retire, None),
            };
            let stamp = self.stamp(index, kind)?;
            let phase = std::mem::replace(&mut self.operations[index].phase, Phase::Active(kind));
            let item = match phase {
                Phase::Read(ticket) => WorkItem::Read(ProviderRead {
                    stamp,
                    ticket,
                    _permit: permit.unwrap(),
                    max_bytes: self.limits.document_bytes,
                }),
                Phase::New(ticket) => WorkItem::PrepareNew(ProviderNew { stamp, ticket }),
                Phase::Retire(mut retired) => {
                    retired.stamp = Some(stamp);
                    WorkItem::Retire(retired)
                }
                _ => unreachable!("queued work"),
            };
            return Ok(Some(item));
        }
        if self.coordinator.next_write().is_none() {
            return Ok(None);
        }
        let permit = self.budget.reserve(self.limits.document_bytes)?;
        let Some(job) = self.coordinator.start_next() else {
            return Ok(None);
        };
        let index = self
            .operations
            .iter()
            .position(|op| op.save == Some(job.token()))
            .expect("issued save operation");
        let stamp = self
            .stamp(index, WorkKind::Write)
            .expect("retained resource");
        self.operations[index].phase = Phase::Active(WorkKind::Write);
        Ok(Some(WorkItem::Write(ProviderWrite {
            stamp,
            job: Box::new(job),
            permit,
            max_bytes: self.limits.document_bytes,
        })))
    }
}
