use crate::{bridge, direct::Region, receipt::Receipt};
#[derive(Clone, Copy)]
pub(crate) enum PacketOp {
    New,
    Call(u64),
    Take(u64),
    Work(u64, u64),
    Complete(u64, u64),
}
#[derive(Clone, Copy)]
pub(crate) enum HandleOp {
    Length,
    BufferFree,
    InputFree,
    AppFree,
    RetireStopped,
}
pub(crate) enum Invocation<'a> {
    Packet(PacketOp, Region<'a>),
    Input(u64, Region<'a>),
    Copy(u64, usize, Region<'a>),
    Handle(HandleOp, u64),
}
impl Invocation<'_> {
    pub fn execute(self) -> bridge::Result<Receipt> {
        let mut r = Receipt::default();
        match self {
            Self::Packet(op, region) => {
                // Safety: borrowed only for this call; host cannot mutate packet concurrently.
                let packet = unsafe { region.read() };
                match op {
                    PacketOp::New => r.app = bridge::app_new(packet)?,
                    PacketOp::Call(app) => {
                        r.reply = bridge::app_call(app, packet)?;
                        r.app = app;
                    }
                    PacketOp::Take(app) => {
                        (r.reply, r.work) = bridge::app_take_work(app, packet)?;
                        r.app = app;
                    }
                    PacketOp::Work(work, input) => {
                        let out = bridge::work_call(work, packet, input)?;
                        r.reply = out.reply;
                        r.result = out.result;
                        r.flags =
                            u32::from(out.work_consumed) | (u32::from(out.input_consumed) << 1);
                        r.work = if out.work_consumed { 0 } else { work };
                        r.input = if out.input_consumed { 0 } else { input };
                    }
                    PacketOp::Complete(app, result) => {
                        r.reply = bridge::app_complete(app, packet, result)?;
                        r.app = app;
                        r.flags = 4;
                    }
                }
            }
            Self::Input(work, source) => {
                // Safety: caller keeps input immutable; bridge makes its bounded owned copy.
                r.input = bridge::work_copy_input(work, unsafe { source.read() })?;
                r.work = work;
            }
            Self::Copy(buffer, offset, destination) => {
                // Safety: destination is writable, exclusive, and disjoint from receipt.
                r.count =
                    bridge::buffer_copy(buffer, offset, unsafe { destination.write() })? as u64;
                r.reply = buffer;
            }
            Self::Handle(op, id) => match op {
                HandleOp::Length => {
                    r.count = bridge::buffer_len(id)? as u64;
                    r.reply = id;
                }
                HandleOp::BufferFree => {
                    bridge::buffer_free(id)?;
                    r.flags = 16;
                }
                HandleOp::InputFree => {
                    bridge::input_free(id)?;
                    r.flags = 2;
                }
                HandleOp::AppFree => {
                    bridge::app_free(id)?;
                    r.flags = 8;
                }
                HandleOp::RetireStopped => {
                    bridge::result_retire_stopped(id)?;
                    r.flags = 4;
                }
            },
        }
        Ok(r)
    }
}
