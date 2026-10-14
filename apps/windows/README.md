# Windows first-stroke host

C# / WinUI 3 native controls call the shared Rust editor through ABI 2. Programmatic
UI uses CommandBar/MenuBar, ColorPicker, NumberBox and HWND-initialized Windows
file pickers. WinUI PathGeometry paints Rust-owned filled paths provisionally;
no permanent renderer decision or GPU performance claim is made (task 002/004).

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
Only x64 is configured and verified by the script's target assumptions; an ARM64
Windows package is not provided. These scripts are local development builds,
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

A Windows-targeted build was attempted on this Mac with EnableWindowsTargeting.
Package restore was still downloading the SDK runtime when source work finished;
no successful WinUI compilation is claimed here. Actual Windows build/run, native
UI inspection, Narrator/accessibility, per-monitor DPI, pen/capture cancellation,
light/dark/narrow layouts, file picker/overwrite/close flows and graphics lifecycle
remain **not run**. This host requires those checks before platform acceptance.
Large-scene rendering is unmeasured; no tablet latency or production performance
claim is made. No dedicated swapchain/device-loss implementation is present: the
provisional XAML path surface lifecycle belongs to WinUI.

References: [WinUI unpackaged deployment](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/unpackage-winui-app),
[pointer input](https://learn.microsoft.com/en-us/windows/apps/develop/input/handle-pointer-input),
[Rust rename semantics](https://doc.rust-lang.org/std/fs/fn.rename.html).
