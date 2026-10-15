//! ABI 2 compatibility guards; current supported 64-bit native ABI layouts.
use sketch_ffi::*;
use std::mem::{align_of, offset_of, size_of};

#[test]
#[cfg(target_pointer_width = "64")]
fn abi2_native_layout_is_unchanged() {
    assert_eq!(sketch_abi_version(), 2);
    assert_eq!(
        (size_of::<SketchSample>(), align_of::<SketchSample>()),
        (48, 8)
    );
    assert_eq!(offset_of!(SketchSample, sample_id), 32);
    assert_eq!(offset_of!(SketchSample, has_pressure), 40);
    assert_eq!(offset_of!(SketchSample, kind), 44);
    assert_eq!(size_of::<SketchBrush>(), 40);
    assert_eq!(offset_of!(SketchBrush, width), 32);
    assert_eq!(size_of::<SketchBounds>(), 32);
    assert_eq!(size_of::<SketchState>(), 64);
    assert_eq!(offset_of!(SketchState, can_undo), 16);
    assert_eq!(offset_of!(SketchState, selection), 32);
    assert_eq!(size_of::<SketchPathCommand>(), 96);
    assert_eq!(offset_of!(SketchPathCommand, verb), 8);
    assert_eq!(offset_of!(SketchPathCommand, red), 16);
    assert_eq!(offset_of!(SketchPathCommand, x1), 48);
    assert_eq!(size_of::<SketchSourceText>(), 16);
    assert_eq!(size_of::<SketchSourceVersion>(), 96);
    assert_eq!(offset_of!(SketchSourceVersion, server_generation), 80);
    assert_eq!(offset_of!(SketchSourceVersion, kind), 88);
}

#[test]
fn abi2_frozen_document_capacity_and_save_revision_contract() {
    const GOLDEN: &[u8] = include_bytes!("../../sketch-core/tests/fixtures/portable-v1.nsketch");
    // Each pointer below addresses live disjoint storage for the call. This test
    // owns the handle and serializes all calls, including its one final free.
    unsafe {
        let mut editor = std::ptr::null_mut();
        assert_eq!(sketch_editor_create(&mut editor), 0);
        assert_eq!(
            sketch_document_import(editor, GOLDEN.as_ptr(), GOLDEN.len()),
            0
        );
        let (mut count, mut revision) = (0, 0);
        let mut bytes = vec![0xa5; GOLDEN.len()];
        assert_eq!(
            sketch_document_export(
                editor,
                bytes.as_mut_ptr(),
                GOLDEN.len() - 1,
                &mut count,
                &mut revision
            ),
            3
        );
        assert_eq!(count, GOLDEN.len());
        assert!(bytes.iter().all(|b| *b == 0xa5));
        assert_eq!(
            sketch_document_export(
                editor,
                bytes.as_mut_ptr(),
                bytes.len(),
                &mut count,
                &mut revision
            ),
            0
        );
        assert_eq!(bytes, GOLDEN);
        assert_eq!(sketch_select_cycle(editor, 1), 0);
        assert_eq!(sketch_delete_selected(editor), 0);
        assert_eq!(sketch_document_mark_saved(editor, revision), 2);
        assert_eq!(sketch_undo(editor), 0);
        assert_eq!(sketch_document_mark_saved(editor, revision), 0);
        assert_eq!(sketch_document_new(editor), 0);
        assert_eq!(sketch_document_mark_saved(editor, revision), 2);
        assert_eq!(
            sketch_document_import(editor, GOLDEN.as_ptr(), GOLDEN.len()),
            0
        );
        assert_eq!(sketch_document_mark_saved(editor, revision), 2);
        sketch_editor_free(editor);
    }
}
