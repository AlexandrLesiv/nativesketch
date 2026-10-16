use crate::{reader::Reader, *};
pub fn decode(bytes: &[u8]) -> Result<Packet, WireError> {
    if bytes.len() > 1024 * 1024 {
        return Err(WireError::capacity());
    }
    let mut header = Reader::new(bytes);
    if header.take::<4>()? != *b"NSAP" {
        return Err(WireError::malformed());
    }
    if header.take::<4>()? != [2, 0, 0, 0] {
        return Err(WireError::transport(4));
    }
    let opcode = header.u32()?;
    let size = header.u32()?;
    let request = header.u64()?;
    if usize::try_from(size).ok().and_then(|n| n.checked_add(24)) != Some(bytes.len()) {
        return Err(WireError::malformed());
    }
    let mut r = Reader::new(&bytes[24..]);
    let command = command(opcode, &mut r)?;
    r.finish()?;
    Ok(Packet {
        opcode,
        request,
        command,
    })
}
fn command(op: u32, r: &mut Reader<'_>) -> Result<Request, WireError> {
    Ok(match op {
        0 => Request::Bootstrap(AppLimits {
            sessions: r.usize()?,
            operations: r.usize()?,
            destinations: r.usize()?,
            buffers: r.usize()?,
            buffer_bytes: r.usize()?,
            document_bytes: r.usize()?,
            storage: StorageLimits {
                jobs: r.usize()?,
                snapshot_bytes: r.usize()?,
            },
        }),
        1 | 8 => {
            let caps = r.u32()?;
            if caps & !3 != 0 {
                return Err(WireError::malformed());
            }
            let config = SessionConfig {
                capabilities: Capabilities {
                    edit_document: caps & 1 != 0,
                    export_document: caps & 2 != 0,
                },
                tool: r.tool()?,
                brush: r.brush()?,
            };
            if op == 1 {
                Request::Create(config)
            } else {
                let policy = match r.u32()? {
                    0 => SaveDestinationPolicy::ReuseBinding,
                    1 => SaveDestinationPolicy::AlwaysChoose,
                    _ => return Err(WireError::malformed()),
                };
                r.zero()?;
                Request::CreateWithSavePolicy(config, policy)
            }
        }
        2 => Request::Detach(r.u64()?),
        3 => Request::Projection(r.u64()?),
        4 => Request::Availability(r.u64()?, r.action()?),
        5 => Request::Stats,
        6 => Request::SelectedBounds(r.u64()?),
        7 => Request::Diagnostic,
        9 => Request::DocumentBounds(r.u64()?),
        10 => Request::Edit(r.context()?, r.edit()?),
        11 => Request::Action(r.context()?, r.action()?),
        12 => Request::Input(r.context()?, PointerId(r.u64()?), r.samples()?),
        20 => {
            let ctx = r.context()?;
            let tag = r.u32()?;
            r.zero()?;
            match tag {
                1 => Request::File(ctx, FileIntent::Save),
                2 => Request::File(ctx, FileIntent::SaveAs),
                3 => Request::File(ctx, FileIntent::ExportCopy),
                4 => Request::File(ctx, FileIntent::Navigate(NavigationIntent::New)),
                5 => Request::File(ctx, FileIntent::Navigate(NavigationIntent::Open)),
                6 => Request::File(ctx, FileIntent::Navigate(NavigationIntent::Close)),
                7 | 8 => Request::Unsupported,
                _ => return Err(WireError::malformed()),
            }
        }
        21 => {
            let ctx = r.context()?;
            let token = r.u64()?;
            let decision = match r.u32()? {
                1 => EffectDecision::Cancel,
                2 => EffectDecision::Unsaved(UnsavedChoice::Save),
                3 => EffectDecision::Unsaved(UnsavedChoice::Discard),
                4 => EffectDecision::Continue,
                5 => EffectDecision::ConfirmClose,
                _ => return Err(WireError::malformed()),
            };
            r.zero()?;
            Request::Reply(ctx, token, decision)
        }
        22 => Request::Resource(r.context()?, r.u64()?, r.resource()?),
        30 => Request::TakeWork,
        31 => Request::Complete,
        32 => Request::Apply(r.u64()?, r.u64()?),
        33 => Request::Cancel(r.u64()?),
        34 => Request::Release,
        40 => Request::Scene(r.u64()?, r.u64()?, r.viewport()?),
        41 => Request::Descriptors(r.u64()?, r.u32()?, r.u32()?),
        42 => Request::GeometryInfo(r.u64()?, r.u64()?),
        43 => Request::GeometryCommands(r.u64()?, r.u64()?, r.u32()?, r.u32()?),
        44 => Request::ReleaseScene(r.u64()?),
        50 => Request::ViewSource(r.context()?, r.u64()?, r.resource()?),
        100 => Request::WorkInfo,
        101 => Request::Encode,
        102 => Request::PrepareRead,
        103 => Request::PrepareNew,
        104 => Request::Finish(r.finish_write()?),
        105 => {
            let failure = Reader::storage_failure(r.u32()?)?;
            r.zero()?;
            Request::Fail(failure)
        }
        106 => Request::Dispose,
        107 => {
            let offset = r.u64()?;
            let maximum = r.u32()?;
            r.zero()?;
            if maximum > 65536 {
                return Err(WireError::capacity());
            }
            Request::CopyEncoded(offset, maximum)
        }
        _ => return Err(WireError::transport(2)),
    })
}
