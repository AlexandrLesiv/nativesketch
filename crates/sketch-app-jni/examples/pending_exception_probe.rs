//! Test-only JVM probe of the actual production export with an already-pending exception.
use jni::{
    EnvUnowned, Outcome,
    objects::{JByteBuffer, JObject, JThrowable},
    sys::{jint, jlong},
};
use std::panic::{AssertUnwindSafe, catch_unwind};
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_nativesketch_portable_PendingExceptionProbe_callWithPendingException<
    'local,
>(
    mut env: EnvUnowned<'local>,
    this: JObject<'local>,
    app: jlong,
    packet: JByteBuffer<'local>,
    length: jlong,
    expected: JThrowable<'local>,
    receipt: JByteBuffer<'local>,
) -> jint {
    catch_unwind(AssertUnwindSafe(|| {
        let outcome = env
            .with_env(|env| -> jni::errors::Result<i32> {
                env.throw(&expected)?;
                // Safety: this is the same attached JVM thread and live caller stack frame;
                // nested JNI entry creates a scoped Env guard, no Rust Env reference escapes.
                let nested = unsafe { EnvUnowned::from_raw(env.get_raw()) };
                Ok(
                    sketch_app_jni::Java_dev_nativesketch_portable_PortableJniV1_appCall(
                        nested, this, app, packet, length, receipt,
                    ),
                )
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
