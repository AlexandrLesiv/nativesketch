use crate::{
    boundary,
    direct::{Region, length_checked},
    invocation::{HandleOp, Invocation},
};
use jni::{
    EnvUnowned,
    objects::{JByteBuffer, JObject},
    sys::{jint, jlong},
};
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_nativesketch_portable_PortableJniV1_bufferLength<'local>(
    env: EnvUnowned<'local>,
    _this: JObject<'local>,
    buffer: jlong,
    receipt: JByteBuffer<'local>,
) -> jint {
    boundary::run(env, &receipt, |_, _, _| {
        Ok(Invocation::Handle(HandleOp::Length, buffer as u64))
    })
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_nativesketch_portable_PortableJniV1_bufferFree<'local>(
    env: EnvUnowned<'local>,
    _this: JObject<'local>,
    buffer: jlong,
    receipt: JByteBuffer<'local>,
) -> jint {
    boundary::run(env, &receipt, |_, _, _| {
        Ok(Invocation::Handle(HandleOp::BufferFree, buffer as u64))
    })
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_nativesketch_portable_PortableJniV1_bufferCopy<'local>(
    env: EnvUnowned<'local>,
    _this: JObject<'local>,
    buffer: jlong,
    offset: jlong,
    destination: JByteBuffer<'local>,
    capacity: jlong,
    receipt: JByteBuffer<'local>,
) -> jint {
    boundary::run(env, &receipt, |env, output, _| {
        let offset = length_checked(offset, isize::MAX as usize)?;
        let destination = Region::resolve(env, &destination, capacity, 65_616, true)?;
        destination.disjoint(output)?;
        Ok(Invocation::Copy(buffer as u64, offset, destination))
    })
}
