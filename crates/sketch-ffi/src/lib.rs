//! Minimal native boundary; the first milestone will add the editor contract.

/// Returns the native interface version expected by the platform shell.
///
/// This function has no allocation, arguments, or panic path. The exported symbol
/// is unique to this library; the canonical declaration is in sketch_ffi.h.
#[unsafe(no_mangle)]
pub extern "C" fn sketch_abi_version() -> u32 {
    1
}
