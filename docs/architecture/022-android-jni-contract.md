# Android Rust JNI transport checkpoint

Implementation contract approved by the lead after the Rust specialist's read-only audit
at `ed53773`. This is an additive transport plan, not implemented Android adoption or native
acceptance. Follow [the host migration plan](022-android-portable-adoption.md); finish the
current browser viewport slice before implementing this checkpoint. Keep native controls,
shared command behavior and document bytes. No C++ replacement is removed before parity.

## API and fixed receipt

Add `sketch-app-jni` over the safe `sketch-app-bridge`, using pinned `jni = "=0.22.4"`
with unnecessary features disabled. Kotlin's private `PortableJniV1` declaration class
exposes `apiVersion(): Int`; require `1` before creating an app. This independently versions
the JNI transport, while packets remain NSAP v2. Use actual generated JVM signatures and
inspect them in tests; never hand-code JNIEnv tables or expose native pointer handles.

Every other native call receives a caller-owned direct receipt buffer last and returns an
integer status. The receipt is exactly 128 bytes; multibyte fields are little-endian:

| Offset | Field |
| --- | --- |
| 0 | status: u32 |
| 4 | consumed flags: u32 |
| 8 | app: u64 |
| 16 | reply/buffer: u64 |
| 24 | work: u64 |
| 32 | input: u64 |
| 40 | result: u64 |
| 48 | count: u64 |
| 56 | existing 72-byte wire error frame |

Consumed bits are work=1, input=2, result=4, app=8, buffer=16. Reserved bits are zero.
Success zeros the error frame and unused fields. Ordinary rejection zeros ownership fields,
flags and count, retains all original caller ownership, and publishes the existing error
and available packet correlation. Zero rejection fields never mean consumed handles.
Kotlin `Long` preserves all opaque u64 bit patterns; no Double conversion is allowed.

The agreed JVM class is file-private `dev.nativesketch.portable.PortableJniV1`, with instance
native methods implementing an internal `NativeCalls` interface. No companion natives,
`@JvmStatic`, overloaded native names or internal-method name mangling. The native receiver
is an instance, not a class. Buffer parameters are nullable `ByteBuffer?` to permit real
null-preflight tests; production wrappers always pass their own private buffers. Explicit
lengths/offsets/capacities are Kotlin `Long`, validated before conversion to Rust sizes.

With `B` expanded to `Ljava/nio/ByteBuffer;`, verify these descriptors against actual
`javap -p -s` output before accepting the exports:

| Method | JVM descriptor shorthand |
| --- | --- |
| apiVersion | `()I` |
| appNew | `(BJB)I` |
| appCall, appTakeWork, workCopyInput | `(JBJB)I` |
| workCall, appComplete | `(JBJJB)I` |
| bufferLength, bufferFree, inputFree, appFree, resultRetireStopped | `(JB)I` |
| bufferCopy | `(JJBJB)I` |

## Calls and successful publication

Each row lists arguments before the receipt and fields populated on success. All fields
not listed are zero; a consumed handle's corresponding field is zero. Return status equals
receipt status when a receipt was published. Wire/body errors still follow existing bridge
semantics; do not reinterpret an actual returned failure result as an unconsumed rejection.

| Call | Successful receipt |
| --- | --- |
| appNew(packet, length) | new app |
| appCall(app, packet, length) | retained app, new reply |
| appTakeWork(app, packet, length) | retained app, new reply, optional issued work |
| workCopyInput(work, source, length) | retained work, new copied input |
| workCall(work, packet, length, input) | new reply, optional actual result, post-call work/input and their consumed bits |
| appComplete(app, packet, length, result) | retained app, new reply, result consumed bit |
| bufferLength(buffer) | retained buffer in reply field, byte count |
| bufferCopy(buffer, offset, destination, capacity) | retained buffer in reply field, copied byte count |
| bufferFree(buffer) | buffer consumed bit |
| inputFree(input) | input consumed bit |
| appFree(app) | app consumed bit, only after actual shutdown readiness |
| resultRetireStopped(result) | result consumed bit, only after actual owner-thread exit |

There is no forced work drop or generic result free. Encoding failure can consume work and
return actual failure proof; rejected completion retains the original result. A live Main
Looper with a detached Activity is not a stopped owner. Prepared scene leases retain their
explicit wire release obligations. App freeing from a finalizer is forbidden.

## Preflight, exclusive buffers and failure boundaries

ByteBuffer position and limit are not transport state: explicit lengths start at native
offset zero, and Kotlin uses absolute access. Inputs may be readonly; receipt and copy
destination must be writable. Before bridge dispatch, validate all references, direct
addresses, writable flags, capacity, nonnegative lengths/offsets, arithmetic overflow and
overlap with writable regions. Cap packets at 1 MiB and copied read input at 64 MiB + 1;
issued work enforces its actual narrower allowance. Bound response-copy chunks at 65,616.

The Kotlin wrapper privately owns these buffers. No other thread may read/write/dispose
writable buffers or mutate input while Rust borrows it. Each owner/worker has its own packet,
receipt and preallocated transfer holder. Address/overlap checks do not prove exclusive
access; document that safety obligation at the JNI boundary and enforce it in the wrapper.
JNI environments and local references do not cross threads. Local references keep memory
alive throughout each call. `workCopyInput` makes the required bounded Rust-owned copy;
host direct-buffer memory has a separate cap and is not included in Rust memory accounting.

Resolve all JNI addresses and readonly queries before mutation. If a Java exception is
already pending, preserve it and return without bridge dispatch. An invalid receipt cannot
carry a valid error record: return rejection before mutation, optionally raising an argument
exception only when no earlier exception is pending. A usable receipt receives normal
argument/transport errors. Do not clear an existing JVM exception to manufacture success.

After an accepted bridge operation, assemble the fixed receipt on the Rust stack and copy
it directly to prevalidated memory. No Java allocation/callback, exception construction or
fallible JNI call may occur between transfer and publication. Kotlin immediately adopts
flags/handles into its existing holder before allocating or parsing a reply. A post-transfer
consumer failure drains known ownership or quarantines it; it must not retry consumed work.
Catch unwinding at every export. Panic status 6 is terminal and distinct from owner-stopped;
JVM OOM or process termination is never reported as successful cleanup.

## Shared scale prerequisite

The shared prerequisite adds `ScaleAnchor::SelectionTopLeft` and portable anchor tag 2.
Center remains tag 1 and the
existing action tag 6/payload size remain unchanged. The native-v1/macOS DTO remains center
only. Resolve the top-left from the committed stroke's transformed bounds, including when
a preview is active. Validate capability, factor, composed geometry and revision/history
capacity before cancelling. Identity scale adds no history; changed scale adds one undo;
repeated relative scale applies again. Request correlation does not imply deduplication.

The shared checkpoint has Rust behavior/validation tests and actual C/Wasm consumers
using `action-scale-top-left.bin` (ANDROID-SCALE-001, INK-001, UNDO-001/002, VIEW-001,
FFI-001). These tests establish the shared boundary, not Android UI acceptance.
Freeze and independently review this small shared-semantic change before Android consumes
the new tag. Include exact packet fixtures and actual C/Wasm consumers; preserve old fixtures.
Checkpoint `d710326` is independently reviewed and integrated as `3f89a49`; the host has
not yet adopted it. Copy-import/nonbinding-save policy remains a separate prerequisite.

## Implementation ownership and acceptance

The lead assigns two isolated writers after this contract: Rust owns the JNI crate, shared
anchor/tests and coordinated workspace manifest/lock changes; Android owns the pure Kotlin
transport/holder/codecs and JVM tests. Agree the exact exported JVM class/method signatures
before concurrent implementation. One assigned owner handles host-JVM evidence and Android
library packaging scripts; no third source writer. Keep every source/test at most 200 lines.

The real host JVM loads the real Rust library through the same Kotlin declarations/wrapper
as Android. Test literal receipt fields/zeros, high-bit values, null/heap/readonly/short or
overlapping buffers, no mutation on every preflight rejection, wrong-thread/owner/kind,
actual worker handoff, consumed encode failure, rejected completion retention, capacity
recovery, prepared-scene ownership, stopped-owner retirement and consumer failure after
adoption. A test-only consumer seam may throw after the call; no production fake-success
command is allowed. Record binary/JVM/source/test identities and all failed attempts.

Then build ARM64/x86-64 Rust JNI libraries and inspect APK contents before removing C++ glue
and stale legacy libraries. Provider orchestration, activity retention, actual MotionEvent/
SAF/focus/TalkBack and per-host Skia adoption are subsequent checkpoints, not inferred from
JVM tests or compilation. No desktop app, emulator, permission change, merge or release is
authorized by this contract beyond the owner's current explicit session permissions.
