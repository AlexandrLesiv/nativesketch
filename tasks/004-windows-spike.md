# 004: Windows native host feasibility

Status: waiting on first accepted macOS slice and renderer decision.
Roles: windows_ui, rendering_performance, reviewer.

On a real Windows development host, create a C# + WinUI 3 shell consuming the shared Rust
library. Validate input batches, DPI changes, resize, resource cleanup, and selected renderer
presentation inside the native UI. Evaluate SwapChainPanel and its composition constraints.

Deliver a built and running Windows prototype with evidence. Compilation elsewhere or
reviewed C# source is not proof of working WinUI/GPU integration. This is a feasibility
spike, not a full Windows feature-parity launch.

