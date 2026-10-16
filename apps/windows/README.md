# Windows first-stroke host

C# / WinUI 3 native controls call the shared Rust editor through ABI 2. Programmatic
UI uses CommandBar/MenuBar, ColorPicker, NumberBox and HWND-initialized Windows
file pickers. WinUI PathGeometry paints Rust-owned filled paths provisionally;
the selected Skia backend has not yet been migrated into this host. No Windows GPU
performance claim is made.

## Build on Windows

Install PowerShell 7, .NET SDK 10, Rust MSVC x64, Visual Studio C++ build tools and
Windows SDK 10.0.19041 or later. From the repository root:

```powershell
./scripts/check-windows.ps1
# Run the built unpackaged app:
./apps/windows/bin/Release/net10.0-windows10.0.19041.0/win-x64/NativeSketch.Windows.exe
```

The project pins Microsoft.WindowsAppSDK 1.8.260921001 and an unpackaged,
self-contained Windows App SDK runtime. The Rust DLL is copied beside the EXE.
The script explicitly builds Rust for `x86_64-pc-windows-msvc` and runs the interop
harness as `win-x64`; an ARM64 Windows package is not provided. These scripts are local development builds,
not release packaging or distribution approval.

## Ownership and behavior

`Engine.cs` is shared verbatim by the app and independent console conformance
harness. Each editor handle has one owner and is disposed once on its creating
thread. All calls, native buffers and native free remain serialized there. There
are no native callbacks. P/Invoke arrays are borrowed for the call; render arrays
are caller-owned. Native status errors become visible exceptions; a caught Rust
panic discards the handle and requires restarting the app. Save/open cancel input
and serialize interaction; synchronous file I/O can block UI on large documents.
Rust performs same-directory temporary-file replacement and owns document format,
pressure, hit testing, transforms, history and validation.

Mouse has absent pressure; pen pressure is forwarded unchanged. Intermediate
frames are deduplicated and ordered. Capture loss, cancellation, Escape, focus
loss, deactivation, resize and XamlRoot changes cancel drafts. DIPs map to document
coordinates through one viewport; per-monitor scale is presentation-only. Layout
changes preserve the document point at the viewport center. Space-drag/scroll pan;
Ctrl-wheel and View commands zoom. Select corners and numeric scale use the same
Rust transaction. Native controls, menu accelerators and next/previous object
commands provide keyboard routes; canvas letter/Delete commands never consume
text-field input. Unsaved replacement and close offer Save/Discard/Cancel.

## Evidence and limits

The independent harness runs on macOS as well as Windows:

```sh
cargo build --release -p sketch-ffi
dotnet run --project apps/windows/InteropCheck -c Release
```

On 2026-10-14, the actual host was macOS arm64 (Darwin 27.0.0), .NET SDK
10.0.107. This harness passed struct layout, mouse/pressure, path extraction,
selection, resize/cancel/undo/redo, delete/undo, ink cancel, Unicode save/reopen,
replacement save, failed-save preservation and corrupt-load preservation.
It uses the exact app wrapper, but **does not certify Windows or WinUI behavior**.

The official SDK packages restored on this Mac. The full build reached Windows-only
`mt.exe` / `MakePri.exe` and failed with executable-format errors, as expected on macOS.
The real C# compiler passed against the WinUI SDK using this compile-only check:

```sh
dotnet msbuild apps/windows/NativeSketch.Windows.csproj -t:Compile \
  -p:Configuration=Release -p:WindowsAppSDKSelfContained=false -v:minimal
```

This command skips Windows resource/deployment packaging; it does not produce a verified
runnable Windows app. The project retains normal self-contained Windows build settings.
Actual Windows build/run, native
UI inspection, Narrator/accessibility, per-monitor DPI, pen/capture cancellation,
light/dark/narrow layouts, file picker/overwrite/close flows and graphics lifecycle
remain **not run**. This host requires those checks before platform acceptance.
Large-scene rendering is unmeasured; no tablet latency or production performance
claim is made. No dedicated swapchain/device-loss implementation is present: the
provisional XAML path surface lifecycle belongs to WinUI.

References: [WinUI unpackaged deployment](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/unpackage-winui-app),
[pointer input](https://learn.microsoft.com/en-us/windows/apps/develop/input/handle-pointer-input),
[Rust rename semantics](https://doc.rust-lang.org/std/fs/fn.rename.html).

## Task 022 composition evidence (2026-10-16)

Tested source: `0e4cd7c5523db69c4a0133652f9eed12c8338646`, extracted from
`2d2a534` without changing the legacy ABI 2 or renderer. `SketchWindow` composes
`EditorView`, `NativeMenus`/`EditorToolbar`, `CanvasInput`, `CanvasNavigation`/
`CanvasViewport`, `CanvasPainter`, `EditorPresenter`, and `DocumentEffects`.
The engine remains uniquely owned by the window. Native event handlers and async
picker/dialog continuations retain the existing DispatcherQueue synchronization
context; no worker or callback transport was introduced. The next portable facade
migration must replace the remaining host-side document/command policy explicitly.

On macOS arm64 (Darwin 27.0.0), .NET 10.0.107 and the pinned WinUI packages:

- The actual WinUI project's `Compile` target passed with no compiler diagnostics,
  using the compile-only command above. This is not Windows packaging or execution.
- `dotnet format <project> --verify-no-changes --no-restore` passed for both projects.
  The WinUI workspace reports an expected macOS `mt.exe` exit 126 warning during
  design-time loading; diagnostics confirm formatting/analyzer execution completed
  with zero changed files. The independent console project has no such warning.
- All 13 handwritten C# files are at most 192 physical lines after formatting.
  Engine and replay lexical tokens are unchanged; all original UI/command string
  literals are retained. Interop layout declarations and assertions are unchanged.
- The existing real C#/Rust replay passed: ABI sizes, mouse/pressure paths, hit test,
  resize cancellation/commit, undo/redo, Unicode repeated save, failed save/load
  preservation, delete/undo and ink cancellation. It loaded the existing macOS
  `libsketch_ffi.dylib`, SHA-256
  `754a8bbfcf6ebb805fded5c8ed6486d49815813544bb4aaaa41b9d8cc1d51c03`.

Relevant preserved invariants are `FFI-001`, `INK-002`, `UNDO-001`, `SAVE-001`,
`LOAD-001`, `FOCUS-001`, `CANCEL-001`, and `VIEW-001`. The console replay covers only
its listed bridge scenarios, not the full native acceptance of those invariants.
No Windows native E2E, FlaUI/UIA3 validation, device/pointer tests, DPI/accessibility,
resource/graphics performance or Windows package launch ran in this slice.
Existing synchronous storage, XAML path rendering and local command policy remain
migration debt, not the final shared Rust/Skia architecture.
