# Portable WebAssembly protocol 1

`sketch-wasm` compiles without JavaScript bindings or external dependencies. All editor semantics and geometry use `sketch-core`. Instantiate with `{}` imports. Use one instance on one JS thread; serialize calls and do not re-enter them. Handles and buffer tokens are nonzero opaque u32 identities, never reused in an instance. Zero from an allocation/create means failure. `sketch_wasm_protocol_version()` returns 1 (independent of C ABI 2).

- `sketch_wasm_create() -> handle`; `sketch_wasm_free(handle) -> status`.
- `sketch_wasm_buffer_new(length) -> token`: initialized writable input bytes.
- `sketch_wasm_buffer_ptr(token) -> byteOffset`, `sketch_wasm_buffer_len(token) -> length`. Unknown token returns zero. Reacquire `memory.buffer` after every call that may allocate; memory growth detaches prior JS views.
- `sketch_wasm_buffer_free(token) -> status`: free caller input exactly once. Outputs belong to the editor and cannot be freed this way (status 2).
- `sketch_wasm_command(handle, opcode, inputToken) -> status`. Zero input token means empty input. Input is copied during the call and may then be freed. Unknown tokens/handles and malformed lengths return 1 without document mutation. Every command invalidates the previous output token (except invalid handle/input rejection). Only successful commands expose new output.
- `sketch_wasm_output(handle) -> outputToken`, or zero when no output. Copy output bytes immediately; treat output memory as read-only. It is invalidated by the next command or editor free. Do not retain JS views across calls.

Status: 0 success, 1 invalid argument/handle/token, 2 invalid state, 3 memory capacity, 4 I/O, 5 corrupt/unsupported document, 6 caught panic. A WASM panic/allocation failure traps: discard the instance; it never unwinds into JS. The raw memory export is a trusted host interface: write only to live allocated input ranges, never arbitrary WASM memory. Each buffer is at most 64 MiB + 8 bytes; live buffer aggregate is at most 128 MiB. Renderer output can reach that limit on large documents; no large-document performance claim is made.

All protocol numbers are little-endian, doubles are IEEE754 binary64. u64 identities must use JavaScript BigInt/DataView getBigUint64/setBigUint64; never convert them to Number. Payload sizes are exact, no native struct padding except explicitly specified below.

| Opcode | Operation | Input | Output |
| --- | --- | --- | --- |
| 1 | State | empty | 64 bytes described below |
| 2 | Begin | pointer u64, device u32, red/green/blue/alpha/width f64 (52 bytes) | empty |
| 3 | Append batch | pointer u64 then zero or more 48-byte samples | empty |
| 4 | Commit | pointer u64 | empty |
| 5 | Cancel gesture | empty | empty |
| 6 | Render | empty | consecutive 96-byte commands |
| 7 | Select at | x/y/tolerance f64 | empty |
| 8 | Cycle selection | signed i32, +1 or -1 | empty |
| 9 | Clear selection | empty | empty |
| 10 | Delete selected | empty | empty |
| 11 | Begin resize | empty | empty |
| 12 | Update resize | scale/anchorX/anchorY f64 | empty |
| 13 | Commit resize | empty | empty |
| 14 | Undo | empty | empty |
| 15 | Redo | empty | empty |
| 16 | New document | empty | empty |
| 17 | Export snapshot | empty | revision u64 then .nsketch bytes |
| 18 | Import | .nsketch bytes | empty |
| 19 | Mark saved | snapshot revision u64 | empty |

State offsets: 0 strokeCount u64; 8 selectedId u64 (zero means none); 16 canUndo u32; 20 canRedo u32; 24 dirty u32; 28 interaction u32 (0 idle, 1 ink, 2 resize); 32/40/48/56 selection x/y/width/height f64.

Sample offsets: 0/8/16/24 x/y/monotonicSeconds/pressure f64; 32 sampleId u64; 40 hasPressure u32 (0 or 1); 44 kind u32 (0 actual, 1 temporary predicted, 2 actual sample correction). Missing pressure must use a finite placeholder, normally zero. Device: 0 mouse, 1 pen, 2 touch. Actual samples remain editable; predictions are never committed or serialized.

Render command offsets: 0 strokeId u64; 8 verb u32; 12 reserved zero u32; 16/24/32/40 RGBA f64; 48/56/64/72/80/88 x1/y1/x2/y2/x3/y3 f64. Verbs 0 move, 1 line, 2 cubic, 3 close. Group contiguous commands by stroke, filling each group with nonzero winding once; no per-subpath alpha accumulation. Coordinates are document-space; host manages viewport mapping. This provisional canvas presentation makes no permanent renderer selection.

Export/import/mark saved require idle state. Import validates before replacing and resets session history only on success. Export does not clear dirty. Mark saved only after the OS confirms writing all bytes successfully; stale acknowledgements return 2. Download initiation cannot confirm durable write and must not clear dirty. Tokens are scoped to the same live editor; do not acknowledge another editor's snapshot. New/import/load invalidate prior document revisions; undo/redo retains revision identity.
