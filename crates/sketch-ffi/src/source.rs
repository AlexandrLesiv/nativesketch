use crate::status::{protect, status};
use crate::{SketchSourceText, SketchSourceVersion};
// Safety: nonempty fields address initialized bytes for their stated lengths;
// caller keeps them alive and immutable through the comparison.
unsafe fn source_text<'a>(text: SketchSourceText) -> Result<&'a str, u32> {
    if text.length > 256 || (text.length != 0 && text.bytes.is_null()) {
        return Err(1);
    }
    if text.length == 0 {
        return Ok("");
    }
    std::str::from_utf8(unsafe { std::slice::from_raw_parts(text.bytes, text.length) })
        .map_err(|_| 1)
}
// Safety: all fields obey source_text's borrowed input contract.
unsafe fn source_version<'a>(
    version: &SketchSourceVersion,
) -> Result<sketch_core::source_policy::Version<'a>, u32> {
    use sketch_core::source_policy::{Kind, Version};
    Ok(Version {
        namespace_id: unsafe { source_text(version.namespace_id)? },
        document_id: unsafe { source_text(version.document_id)? },
        revision_id: unsafe { source_text(version.revision_id)? },
        parent_revision_id: unsafe { source_text(version.parent_revision_id)? },
        content_sha256: unsafe { source_text(version.content_sha256)? },
        server_generation: version.server_generation,
        kind: match version.kind {
            0 => Kind::Draft,
            1 => Kind::Published,
            _ => return Err(1),
        },
    })
}
/// Compare metadata without modifying a document or retaining any pointers.
/// # Safety
/// Current may be null. Candidate must address a valid struct. All nonempty text
/// fields address initialized bytes, immutable and alive throughout the call.
/// Output must address writable u32 storage disjoint from all borrowed inputs.
/// May run concurrently on any thread if those memory rules are respected.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_source_compare(
    current: *const SketchSourceVersion,
    candidate: *const SketchSourceVersion,
    out_decision: *mut u32,
) -> u32 {
    protect(|| {
        if out_decision.is_null() {
            return Err(1);
        }
        let candidate = unsafe { candidate.as_ref() }.ok_or(1u32)?;
        let candidate = unsafe { source_version(candidate)? };
        let current = unsafe { current.as_ref() }
            .map(|v| unsafe { source_version(v) })
            .transpose()?;
        let decision =
            sketch_core::source_policy::compare(current.as_ref(), &candidate).map_err(status)?;
        unsafe {
            *out_decision = decision as u32;
        }
        Ok(())
    })
}
