//! Native C-ABI fixture generator for the independent Node/WASM conformance check.
use sketch_ffi::*;
fn main() {
    let path = std::env::args().nth(1).expect("fixture path");
    unsafe {
        let mut e = std::ptr::null_mut();
        assert_eq!(sketch_editor_create(&mut e), 0);
        assert_eq!(
            sketch_stroke_begin(
                e,
                42,
                1,
                SketchBrush {
                    red: 0.2,
                    green: 0.3,
                    blue: 0.4,
                    alpha: 0.5,
                    width: 6.0
                }
            ),
            0
        );
        let samples = [
            SketchSample {
                x: 10.0,
                y: 20.0,
                timestamp: 1.0,
                pressure: 0.5,
                sample_id: 1,
                has_pressure: 1,
                kind: 0,
            },
            SketchSample {
                x: 30.0,
                y: 40.0,
                timestamp: 2.0,
                pressure: 0.7,
                sample_id: 2,
                has_pressure: 1,
                kind: 0,
            },
            SketchSample {
                x: 80.0,
                y: 90.0,
                timestamp: 3.0,
                pressure: 0.9,
                sample_id: 3,
                has_pressure: 1,
                kind: 1,
            },
        ];
        assert_eq!(
            sketch_stroke_append(e, 42, samples.as_ptr(), samples.len()),
            0
        );
        assert_eq!(sketch_stroke_commit(e, 42), 0);
        let (mut n, mut rev) = (0, 0);
        assert_eq!(
            sketch_document_export(e, std::ptr::null_mut(), 0, &mut n, &mut rev),
            0
        );
        let mut bytes = vec![0; n];
        assert_eq!(
            sketch_document_export(e, bytes.as_mut_ptr(), n, &mut n, &mut rev),
            0
        );
        std::fs::write(path, bytes).unwrap();
        sketch_editor_free(e);
    }
}
