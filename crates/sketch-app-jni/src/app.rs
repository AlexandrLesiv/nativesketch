use crate::{
    boundary,
    invocation::{HandleOp, Invocation, PacketOp},
};
use jni::{
    EnvUnowned,
    objects::{JByteBuffer, JObject},
    sys::{jint, jlong},
};
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_nativesketch_portable_PortableJniV1_apiVersion(
    _env: EnvUnowned<'_>,
    _this: JObject<'_>,
) -> jint {
    1
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_nativesketch_portable_PortableJniV1_appNew<'local>(
    env: EnvUnowned<'local>,
    _this: JObject<'local>,
    packet: JByteBuffer<'local>,
    length: jlong,
    receipt: JByteBuffer<'local>,
) -> jint {
    boundary::packet(env, &receipt, &packet, length, PacketOp::New)
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_nativesketch_portable_PortableJniV1_appCall<'local>(
    env: EnvUnowned<'local>,
    _this: JObject<'local>,
    app: jlong,
    packet: JByteBuffer<'local>,
    length: jlong,
    receipt: JByteBuffer<'local>,
) -> jint {
    boundary::packet(env, &receipt, &packet, length, PacketOp::Call(app as u64))
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_nativesketch_portable_PortableJniV1_appTakeWork<'local>(
    env: EnvUnowned<'local>,
    _this: JObject<'local>,
    app: jlong,
    packet: JByteBuffer<'local>,
    length: jlong,
    receipt: JByteBuffer<'local>,
) -> jint {
    boundary::packet(env, &receipt, &packet, length, PacketOp::Take(app as u64))
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_nativesketch_portable_PortableJniV1_appComplete<'local>(
    env: EnvUnowned<'local>,
    _this: JObject<'local>,
    app: jlong,
    packet: JByteBuffer<'local>,
    length: jlong,
    result: jlong,
    receipt: JByteBuffer<'local>,
) -> jint {
    boundary::packet(
        env,
        &receipt,
        &packet,
        length,
        PacketOp::Complete(app as u64, result as u64),
    )
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_nativesketch_portable_PortableJniV1_appFree<'local>(
    env: EnvUnowned<'local>,
    _this: JObject<'local>,
    app: jlong,
    receipt: JByteBuffer<'local>,
) -> jint {
    boundary::run(env, &receipt, |_, _, _| {
        Ok(Invocation::Handle(HandleOp::AppFree, app as u64))
    })
}
