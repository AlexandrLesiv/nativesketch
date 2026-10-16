# Legacy Apple bridge declaration

`CSketchFFI/module.modulemap` exposes the existing Rust ABI 2 header to UIKit and
read-only Apple viewers. It contains no C implementation and does not depend on the
macOS source tree. The build scripts link each target's `libsketch_ffi.a` explicitly.

This compatibility boundary remains until these hosts migrate to the portable Rust
application contracts. Production macOS uses its distinct versioned session module.
