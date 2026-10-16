use crate::{
    boundary,
    direct::Region,
    invocation::{HandleOp, Invocation, PacketOp},
};
use jni::{
    EnvUnowned,
    objects::{JByteBuffer, JObject},
    sys::{jint, jlong},
};
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_nativesketch_portable_PortableJniV1_workCopyInput<'local>(
    env: EnvUnowned<'local>,
    _this: JObject<'local>,
    work: jlong,
    source: JByteBuffer<'local>,
    length: jlong,
    receipt: JByteBuffer<'local>,
) -> jint {
    boundary::run(env, &receipt, |env, output, _| {
        let input = Region::resolve(env, &source, length, 64 * 1024 * 1024 + 1, false)?;
        input.disjoint(output)?;
        Ok(Invocation::Input(work as u64, input))
    })
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_nativesketch_portable_PortableJniV1_workCall<'local>(
    env: EnvUnowned<'local>,
    _this: JObject<'local>,
    work: jlong,
    packet: JByteBuffer<'local>,
    length: jlong,
    input: jlong,
    receipt: JByteBuffer<'local>,
) -> jint {
    boundary::packet(
        env,
        &receipt,
        &packet,
        length,
        PacketOp::Work(work as u64, input as u64),
    )
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_nativesketch_portable_PortableJniV1_inputFree<'local>(
    env: EnvUnowned<'local>,
    _this: JObject<'local>,
    input: jlong,
    receipt: JByteBuffer<'local>,
) -> jint {
    boundary::run(env, &receipt, |_, _, _| {
        Ok(Invocation::Handle(HandleOp::InputFree, input as u64))
    })
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_nativesketch_portable_PortableJniV1_resultRetireStopped<'local>(
    env: EnvUnowned<'local>,
    _this: JObject<'local>,
    result: jlong,
    receipt: JByteBuffer<'local>,
) -> jint {
    boundary::run(env, &receipt, |_, _, _| {
        Ok(Invocation::Handle(HandleOp::RetireStopped, result as u64))
    })
}
