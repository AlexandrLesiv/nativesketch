# 004: Windows native host feasibility

Status: waiting on first accepted macOS slice and renderer decision.
Roles: windows_ui, rendering_performance, reviewer.

On a real Windows development host, create a C# + WinUI 3 shell consuming the shared Rust
library. Validate input batches, DPI changes, resize, resource cleanup, and selected renderer
presentation inside the native UI. Evaluate SwapChainPanel and its composition constraints.

Deliver a built and running Windows prototype with evidence. Compilation elsewhere or
reviewed C# source is not proof of working WinUI/GPU integration. This is a feasibility
spike, not a full Windows feature-parity launch.


## Existing prototype and remaining port work

The WinUI/C# host now exists (task005), but compilation and C#↔Rust conformance on
macOS did not validate Windows packaging or native runtime behavior. Continue that
implementation; do not create a duplicate Windows host.

Acceptance on Windows includes actual WinUI launch/packaging, accepted renderer
SwapChainPanel composition, mixed-DPI/multi-monitor and resize/surface-loss behavior,
mouse/pen/keyboard plus text-field isolation, Narrator, native picker permission and
failure retention, and portable Rust document roundtrips. Resolve two-pointer navigation
versus cancellation explicitly. Report x64/ARM64 support only for built/tested targets.
Deliver native designer assessment and fresh independent review before owner merge
approval. Publishing/signing policy and crash recovery require explicit scope.
