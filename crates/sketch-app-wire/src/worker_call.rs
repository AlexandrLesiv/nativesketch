use crate::{work::WorkState, *};
impl WireWork {
    /// Invalid packets/scope/outcomes retain work and input. Accepted completion consumes once.
    pub fn call(
        &mut self,
        bytes: &[u8],
        input: &mut Option<InputBytes>,
    ) -> Result<WorkerReply, WireError> {
        let packet = decode(bytes)?;
        let maximum = match packet.command {
            Request::CopyEncoded(_, n) => 80 + n as usize,
            _ => 4096,
        };
        let mut reply = Reply::reserve(&self.budget, packet.opcode, packet.request, maximum)?;
        self.validate_call(&packet.command, input)?;
        let identity = crate::identity::reserve(1)?;
        let result = match packet.command {
            Request::WorkInfo => {
                self.info_reply(&mut reply)?;
                None
            }
            Request::CopyEncoded(offset, maximum) => {
                let Some(WorkState::Encoded(work)) = &self.state else {
                    unreachable!()
                };
                let offset = usize::try_from(offset).map_err(|_| WireError::malformed())?;
                let bytes = work.bytes().get(offset..).ok_or(WireError::malformed())?;
                let count = bytes.len().min(maximum as usize);
                reply.u32(u32::try_from(count).expect("page limit"));
                reply.u32(0);
                reply.raw(&bytes[..count]);
                None
            }
            Request::Encode => {
                let Some(WorkState::Issued(WorkItem::Write(work))) = self.state.take() else {
                    unreachable!()
                };
                match work.encode() {
                    Ok(work) => {
                        reply.u32(0);
                        reply.u32(0);
                        reply.u64(work.bytes().len() as u64);
                        self.state = Some(WorkState::Encoded(work));
                        None
                    }
                    Err(result) => {
                        reply.u32(1);
                        reply.u32(0);
                        reply.u64(0);
                        Some(result)
                    }
                }
            }
            Request::Finish(outcome) => {
                let Some(WorkState::Encoded(work)) = self.state.take() else {
                    unreachable!()
                };
                match work.finish(outcome) {
                    Ok(result) => Some(result),
                    Err((error, work)) => {
                        self.state = Some(WorkState::Encoded(work));
                        return Err(error.into());
                    }
                }
            }
            Request::PrepareRead => {
                let Some(WorkState::Issued(WorkItem::Read(work))) = self.state.take() else {
                    unreachable!()
                };
                Some(work.prepare(input.take().expect("validated input").bytes))
            }
            Request::PrepareNew => {
                let Some(WorkState::Issued(WorkItem::PrepareNew(work))) = self.state.take() else {
                    unreachable!()
                };
                Some(work.prepare())
            }
            Request::Dispose => {
                let Some(WorkState::Issued(WorkItem::Retire(work))) = self.state.take() else {
                    unreachable!()
                };
                Some(work.dispose())
            }
            Request::Fail(failure) => match self.state.take() {
                Some(WorkState::Issued(WorkItem::Write(work))) => Some(work.fail(failure)),
                Some(WorkState::Issued(WorkItem::Read(work))) => Some(work.fail(failure)),
                _ => unreachable!(),
            },
            _ => unreachable!("validated scope"),
        };
        Ok(WorkerReply {
            reply: reply.finish(),
            result: result.map(|result| WireResult {
                app: self.app,
                operation: self.operation,
                inner: Some(result),
                _identity: identity,
            }),
        })
    }
    fn validate_call(
        &self,
        command: &Request,
        input: &Option<InputBytes>,
    ) -> Result<(), WireError> {
        if self.state.is_none() {
            return Err(WireError::plain(2));
        }
        if !matches!(command, Request::PrepareRead) && input.is_some() {
            return Err(WireError::malformed());
        }
        let valid = match (command, &self.state) {
            (Request::WorkInfo, _) => true,
            (Request::Encode, Some(WorkState::Issued(WorkItem::Write(_)))) => true,
            (Request::PrepareRead, Some(WorkState::Issued(WorkItem::Read(_)))) => {
                let input = input.as_ref().ok_or(WireError::malformed())?;
                if input.app != self.app {
                    return Err(WireError::plain(11));
                }
                true
            }
            (Request::PrepareNew, Some(WorkState::Issued(WorkItem::PrepareNew(_)))) => true,
            (Request::Dispose, Some(WorkState::Issued(WorkItem::Retire(_)))) => true,
            (Request::Finish(_) | Request::CopyEncoded(..), Some(WorkState::Encoded(_))) => true,
            (Request::Fail(_), Some(WorkState::Issued(WorkItem::Read(_) | WorkItem::Write(_)))) => {
                true
            }
            _ => false,
        };
        if valid {
            Ok(())
        } else {
            Err(WireError::transport(3))
        }
    }
}
