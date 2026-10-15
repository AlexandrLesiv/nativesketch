use crate::status::protect;
use crate::*;
use std::ptr;
#[test]
fn byte_buffers_have_owned_copy_and_atomic_capacity_errors() {
    unsafe {
        let mut e = ptr::null_mut();
        assert_eq!(sketch_editor_create(&mut e), 0);
        let mut n = 0;
        let mut rev = 0;
        assert_eq!(
            sketch_document_export(e, ptr::null_mut(), 0, &mut n, &mut rev),
            0
        );
        let mut small = [255u8; 1];
        assert_eq!(
            sketch_document_export(e, small.as_mut_ptr(), 1, &mut n, &mut rev),
            3
        );
        assert_eq!(small, [255]);
        assert_eq!(
            sketch_document_export(e, ptr::null_mut(), 1, &mut n, &mut rev),
            1
        );
        let mut bytes = vec![0; n];
        assert_eq!(
            sketch_document_export(e, bytes.as_mut_ptr(), bytes.len(), &mut n, &mut rev),
            0
        );
        assert_eq!(sketch_document_import(e, ptr::null(), 0), 1);
        assert_eq!(sketch_document_import(e, b"bad".as_ptr(), 3), 5);
        assert_eq!(sketch_document_mark_saved(e, rev), 0);
        assert_eq!(sketch_document_import(e, bytes.as_ptr(), bytes.len()), 0);
        bytes.fill(0);
        assert_eq!(sketch_document_mark_saved(e, rev), 2);
        assert_eq!(
            sketch_document_export(e, ptr::null_mut(), 0, &mut n, &mut rev),
            0
        );
        assert_eq!(n, 24);
        sketch_editor_free(e);
    }
}

#[test]
fn panic_is_contained() {
    assert_eq!(protect(|| panic!("injected panic")), 6);
}
#[test]
fn ffi_errors_copy_ownership_and_real_editor_flow() {
    unsafe {
        assert_eq!(sketch_editor_create(ptr::null_mut()), 1);
        assert_eq!(sketch_cancel(ptr::null_mut()), 1);
        let mut e = ptr::null_mut();
        assert_eq!(sketch_editor_create(&mut e), 0);
        let b = SketchBrush {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
            width: 4.0,
        };
        assert_eq!(sketch_stroke_begin(e, 42, 0, b), 0);
        assert_eq!(sketch_stroke_append(e, 42, ptr::null(), 1), 1);
        let mut sample = SketchSample {
            x: 12.0,
            y: 20.0,
            timestamp: 1.0,
            pressure: 0.0,
            sample_id: 1,
            has_pressure: 2,
            kind: 0,
        };
        assert_eq!(sketch_stroke_append(e, 42, &sample, 1), 1);
        sample.has_pressure = 0;
        assert_eq!(sketch_stroke_append(e, 42, &sample, 1), 0);
        sample.x = 500.0;
        assert_eq!(sample.x, 500.0);
        assert_eq!(sketch_stroke_commit(e, 42), 0);
        let mut count = 0;
        assert_eq!(sketch_render_copy(e, ptr::null_mut(), 0, &mut count), 0);
        assert!(count > 0);
        let mut small = SketchPathCommand {
            stroke_id: 999,
            ..Default::default()
        };
        assert_eq!(sketch_render_copy(e, &mut small, 1, &mut count), 3);
        assert_eq!(small.stroke_id, 999);
        let mut output = vec![SketchPathCommand::default(); count];
        assert_eq!(
            sketch_render_copy(e, output.as_mut_ptr(), output.len(), &mut count),
            0
        );
        assert!(output.iter().all(|c| c.x1 < 100.0));
        let mut state = SketchState::default();
        assert_eq!(sketch_editor_state(e, &mut state), 0);
        assert_eq!(state.stroke_count, 1);
        assert_eq!(sketch_undo(e), 0);
        assert_eq!(sketch_redo(e), 0);
        assert_eq!(sketch_document_load(e, b"\xff".as_ptr(), 1), 1);
        assert_eq!(sketch_editor_state(e, ptr::null_mut()), 1);
        sketch_editor_free(e);
        sketch_editor_free(ptr::null_mut());
    }
}
