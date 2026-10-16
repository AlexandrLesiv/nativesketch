# Portable JNI transport v1

`sketch_app_jni` is a host-policy-free cdylib over the safe `sketch-app-bridge`.
Its instance exports match `dev.nativesketch.portable.PortableJniV1`; the authoritative
signatures and 128-byte receipt layout are in
[`022-android-jni-contract.md`](../../docs/architecture/022-android-jni-contract.md).

The Kotlin owner must privately retain its direct buffers and prevent concurrent access
for the entire call. JNI local references pin the buffers; Rust does not retain a pointer
beyond the call. An immutable input region is borrowed only after validation, and copying
provider input creates a separately bounded Rust-owned allocation in the safe bridge.
Address/overlap validation alone does not prevent another Java thread from writing memory.
Offsets and explicit lengths ignore ByteBuffer position/limit. Receipts need at least
128 writable bytes; publication writes exactly the first 128. No memory is published when
the receipt is invalid or an exception is already pending. Return status then does not
promise a valid receipt. Negative lengths reject with 1; bounded size-limit excess uses 3.

Preflight resolves all JNI references, capacities and writability before invoking a bridge
operation. A readable exact packet carries its correlation even on overlap rejection.
After a successful bridge transfer, only fixed Rust stack assembly and direct receipt
publication remain. The Kotlin wrapper must adopt ownership immediately before any reply
parsing/allocation; consumed resources cannot be retried after consumer failure. Panics
return terminal status 6, while actual stopped-owner retirement remains a distinct bridge
operation. Unexpected loss of publication is a quarantine situation, not a retry signal.

The ordinary dylib contains no test helper exports. `cargo build -p sketch-app-jni
--example pending_exception_probe` builds a separate test-only dylib with the exact JNI
implementation plus a pending-exception probe. Load that artifact **alone in a fresh JVM**:
it has its own Rust registry and cannot share handles with the ordinary dylib. The probe
throws the supplied Throwable and calls the production `appCall` implementation; the JVM
test checks object identity, unchanged receipt bytes and unchanged application state.
Never package the probe in an APK. Artifact hashes and JVM execution results are separate
from Android device acceptance. Unit tests in this crate are host-scoped; receipt/range
checks do not establish native UI behavior.
