# 0001: Native applications around a Rust engine

Status: accepted foundation; renderer and document format remain open.
Date: 2026-10-14

The owner selected infinite vector sketching, strict native controls, a solo/agent-assisted
workflow, Swift/macOS first, and C# + WinUI 3 for Windows. Native input and OS integration
remain platform-specific; geometry and document semantics are shared in Rust.

The repository begins with a minimal AppKit executable linked to a Rust static library.
This verifies a real native boundary without selecting a renderer prematurely. The initial
ABI has a version handshake only. SwiftPM and macOS 14 are bootstrap choices, not a final
deployment commitment. An Xcode application target can replace the development bundle
when entitlements, signing, document lifecycle, or distribution require it.

Agents work through Codex with project-local Astra configurations. The owner approves
merges. UI/UX design is a first-class role before implementation and during running-app review.
Keep the initial orchestration small and evaluate it against accepted changes.

