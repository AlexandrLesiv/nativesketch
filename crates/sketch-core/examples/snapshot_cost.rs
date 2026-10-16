//! Synchronous snapshot-copy cost only: no native input, UI, storage or pen latency claim.
use sketch_core::application::storage::{
    DestinationId, SaveCoordinator, SavePurpose, StorageLimits,
};
use sketch_core::application::{Capabilities, EditorSession, RequestId, ToolId, ValidatedDocument};
use sketch_core::{Brush, Editor, Input, Sample};
use std::time::Instant;
fn main() {
    let brush = Brush {
        rgba: [0.0, 0.0, 0.0, 1.0],
        width: 3.0,
    };
    for count in [1_000usize, 10_000, 100_000, 1_000_000] {
        let mut source = Editor::new();
        for start in (0..count).step_by(100_000) {
            source.begin(1, 1, brush).unwrap();
            let samples: Vec<_> = (0..(count - start).min(100_000))
                .map(|index| {
                    Input::Actual(Sample {
                        id: index as u64 + 1,
                        x: (index % 1000) as f64,
                        y: 20.0,
                        timestamp: index as f64,
                        pressure: Some(0.5),
                    })
                })
                .collect();
            source.append(1, &samples).unwrap();
            source.commit(1).unwrap();
        }
        let bytes = source.export().unwrap().0;
        let mut session = EditorSession::new(
            Capabilities {
                edit_document: true,
                export_document: true,
            },
            ToolId::Pen,
            brush.try_into().unwrap(),
            Some(ValidatedDocument::decode(&bytes).unwrap()),
        )
        .unwrap();
        let mut timings = Vec::new();
        let mut charged = 0;
        for iteration in 0..21 {
            let mut queue = SaveCoordinator::new(StorageLimits {
                jobs: 1,
                snapshot_bytes: 128 * 1024 * 1024,
            })
            .unwrap();
            let started = Instant::now();
            queue
                .request(
                    &mut session,
                    RequestId(iteration),
                    SavePurpose::ExportCopy,
                    DestinationId(1),
                )
                .unwrap();
            std::hint::black_box(&queue);
            let micros = started.elapsed().as_secs_f64() * 1_000_000.0;
            charged = queue.reserved_snapshot_bytes();
            // Consume and verify the actual snapshot outside the timed copy interval.
            assert_eq!(queue.start_next().unwrap().encode().unwrap(), bytes);
            if iteration > 0 {
                timings.push(micros);
            }
        }
        timings.sort_by(f64::total_cmp);
        println!(
            "{{\"samples\":{count},\"encoded_bytes\":{},\"charged_snapshot_bytes\":{charged},\"iterations\":20,\"median_us\":{:.3},\"max_us\":{:.3}}}",
            bytes.len(),
            (timings[9] + timings[10]) / 2.0,
            timings[19]
        );
    }
}
