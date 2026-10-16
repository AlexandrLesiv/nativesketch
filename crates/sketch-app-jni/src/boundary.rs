use crate::{
    bridge,
    direct::Region,
    invocation::{Invocation, PacketOp},
    receipt::{Receipt, error},
};
use jni::{Env, EnvUnowned, Outcome, objects::JByteBuffer};
use std::panic::{AssertUnwindSafe, catch_unwind};
pub(crate) fn run<'local, 'buffers>(
    mut env: EnvUnowned<'local>,
    receipt: &'buffers JByteBuffer<'local>,
    preflight: impl FnOnce(
        &mut Env<'local>,
        Region<'buffers>,
        &mut (u32, u64),
    ) -> bridge::Result<Invocation<'buffers>>,
) -> i32 {
    // Covers attachment/preflight as well as the bridge's own panic containment.
    catch_unwind(AssertUnwindSafe(|| {
        let outcome = env
            .with_env(|env| -> jni::errors::Result<i32> {
                if env.exception_check() {
                    return Ok(1);
                }
                let output = match Region::resolve(env, receipt, 128, 128, true) {
                    Ok(output) => output,
                    Err(_) => return Ok(1), // no usable receipt: no publication or dispatch
                };
                let mut correlation = (0, 0);
                let prepared = preflight(env, output, &mut correlation);
                // Every fallible JNI call ends here, strictly before any ownership transfer.
                if env.exception_check() {
                    return Ok(1);
                }
                let result = match prepared {
                    Ok(call) => guarded(|| call.execute()),
                    Err(e) => Err(e),
                };
                let (status, bytes) = Receipt::bytes(result, correlation);
                // Safety: preflighted writable extent, held local ref, disjoint input/output.
                unsafe { output.publish(&bytes) };
                Ok(status)
            })
            .into_outcome();
        match outcome {
            Outcome::Ok(status) => status,
            Outcome::Err(_) => 1,
            Outcome::Panic(_) => 6,
        }
    }))
    .unwrap_or(6)
}
pub(crate) fn guarded(f: impl FnOnce() -> bridge::Result<Receipt>) -> bridge::Result<Receipt> {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or_else(|_| Err(error(6)))
}
pub(crate) fn packet<'local>(
    env: EnvUnowned<'local>,
    receipt: &JByteBuffer<'local>,
    source: &JByteBuffer<'local>,
    length: i64,
    operation: PacketOp,
) -> i32 {
    run(env, receipt, |env, output, correlation| {
        let input = Region::resolve(env, source, length, 1024 * 1024, false)?;
        // Safety: input is readable and pinned; no writable Rust borrow or publication
        // exists yet. End this read before rejecting overlap and publishing the error.
        *correlation = bridge::correlation(unsafe { input.read() });
        input.disjoint(output)?;
        Ok(Invocation::Packet(operation, input))
    })
}
