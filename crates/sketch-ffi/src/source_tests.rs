use crate::*;
use std::ptr;
fn text(s: &'static str) -> SketchSourceText {
    SketchSourceText {
        bytes: s.as_ptr(),
        length: s.len(),
    }
}
fn version() -> SketchSourceVersion {
    SketchSourceVersion {
        namespace_id: text("account/library"),
        document_id: text("drawing"),
        revision_id: text("revision"),
        parent_revision_id: SketchSourceText {
            bytes: ptr::null(),
            length: 0,
        },
        content_sha256: text("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        server_generation: 0,
        kind: 0,
    }
}
#[test]
fn ffi_source_comparison_validates_buffers_and_preserves_output_on_failure() {
    let valid = version();
    let mut decision = 999;
    unsafe {
        assert_eq!(sketch_source_compare(ptr::null(), &valid, &mut decision), 0);
        assert_eq!(decision, 0);
        assert_eq!(sketch_source_compare(&valid, &valid, &mut decision), 0);
        assert_eq!(decision, 1);
        assert_eq!(
            sketch_source_compare(ptr::null(), ptr::null(), &mut decision),
            1
        );
        assert_eq!(
            sketch_source_compare(ptr::null(), &valid, ptr::null_mut()),
            1
        );
        assert_eq!(decision, 1);
        let mut invalid = valid;
        invalid.kind = 2;
        assert_eq!(
            sketch_source_compare(ptr::null(), &invalid, &mut decision),
            1
        );
        invalid = valid;
        invalid.namespace_id = SketchSourceText {
            bytes: ptr::null(),
            length: 1,
        };
        assert_eq!(
            sketch_source_compare(ptr::null(), &invalid, &mut decision),
            1
        );
        invalid.namespace_id = SketchSourceText {
            bytes: ptr::null(),
            length: 257,
        };
        assert_eq!(
            sketch_source_compare(ptr::null(), &invalid, &mut decision),
            1
        );
        invalid.namespace_id = SketchSourceText {
            bytes: b"\xff".as_ptr(),
            length: 1,
        };
        assert_eq!(
            sketch_source_compare(ptr::null(), &invalid, &mut decision),
            1
        );
        assert_eq!(sketch_source_compare(&invalid, &valid, &mut decision), 1);
        invalid.namespace_id = text("account\nother");
        assert_eq!(
            sketch_source_compare(ptr::null(), &invalid, &mut decision),
            1
        );
        assert_eq!(decision, 1);
    }
}
#[test]
fn ffi_same_content_revision_advance_prevents_rollback_and_keeps_ancestry() {
    let r1 = SketchSourceVersion {
        kind: 1,
        server_generation: 1,
        ..version()
    };
    let r2 = SketchSourceVersion {
        revision_id: text("r2"),
        parent_revision_id: r1.revision_id,
        content_sha256: text("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        server_generation: 2,
        ..r1
    };
    let r3 = SketchSourceVersion {
        revision_id: text("r3"),
        parent_revision_id: r2.revision_id,
        server_generation: 3,
        ..r1
    };
    let mut cached = r1;
    let mut decision = 999;
    unsafe {
        assert_eq!(sketch_source_compare(&cached, &r3, &mut decision), 0);
        assert_eq!(decision, 0);
        cached = r3;
        assert_eq!(sketch_source_compare(&cached, &r2, &mut decision), 0);
        assert_eq!(decision, 2);
        let d1 = SketchSourceVersion {
            kind: 0,
            revision_id: text("d1"),
            parent_revision_id: r3.revision_id,
            ..r3
        };
        let d2 = SketchSourceVersion {
            revision_id: text("d2"),
            parent_revision_id: d1.revision_id,
            content_sha256: r2.content_sha256,
            ..d1
        };
        assert_eq!(sketch_source_compare(&cached, &d1, &mut decision), 0);
        assert_eq!(decision, 0);
        cached = d1;
        assert_eq!(sketch_source_compare(&cached, &d2, &mut decision), 0);
        assert_eq!(decision, 0);
    }
}
#[test]
fn ffi_policy_reports_nonmutating_conflicts_and_identity_mismatch() {
    let current = version();
    let mut candidate = current;
    let mut decision = 999;
    unsafe {
        candidate.namespace_id = text("another-account/library");
        assert_eq!(
            sketch_source_compare(&current, &candidate, &mut decision),
            0
        );
        assert_eq!(decision, 4);
        candidate = current;
        candidate.revision_id = text("another-revision");
        candidate.content_sha256 =
            text("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
        assert_eq!(
            sketch_source_compare(&current, &candidate, &mut decision),
            0
        );
        assert_eq!(decision, 3);
        candidate.parent_revision_id = current.revision_id;
        assert_eq!(
            sketch_source_compare(&current, &candidate, &mut decision),
            0
        );
        assert_eq!(decision, 0);
        assert_eq!(
            sketch_source_compare(&candidate, &current, &mut decision),
            0
        );
        assert_eq!(decision, 2);
    }
}
