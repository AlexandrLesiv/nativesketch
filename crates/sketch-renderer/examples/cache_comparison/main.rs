//! CPU-only cache experiment. It never constructs a Metal context or native window.
mod fixture;
mod measure;
use fixture::Fixture;
use measure::Measurement;
use sketch_core::application::{DeviceKind, EditorCommand, PointerId, ToolId};
use std::{io, path::Path, time::Duration};
fn main() -> io::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 4
        || !args[2..]
            .iter()
            .all(|value| !value.is_empty() && value.chars().all(|ch| ch.is_ascii_hexdigit()))
    {
        return Err(io::Error::other(
            "usage: cache_comparison OUTPUT.csv SOURCE_REVISION BINARY_SHA256",
        ));
    }
    let path = Path::new(&args[1]);
    let metadata = format!(
        concat!(
            "{{\n  \"revision\": \"{}\",\n  \"binary_sha256\": \"{}\",\n",
            "  \"renderer\": \"Skia 0.153.3 CPU raster; no Metal or native presentation\",\n",
            "  \"build\": \"{}\",\n  \"canvas_pixels\": [1024, 768],\n",
            "  \"fixture\": \"128 strokes, 32 pressure samples each; deterministic fixture.rs\",\n",
            "  \"scene_policy\": \"one shared prepared snapshot per pair, geometry cache enabled\",\n",
            "  \"reference_policy\": \"Skia path and background caches bypassed\",\n",
            "  \"pair_order\": \"alternating cached/reference then reference/cached\",\n",
            "  \"verification\": \"exact full RGBA comparison after every measured pair\",\n",
            "  \"warmup_pairs\": 5,\n  \"cold_pairs\": 10,\n  \"warm_pairs\": 32,\n",
            "  \"after_idle_pairs\": 4,\n  \"idle_ms\": 250,\n  \"live_pairs\": 32,\n",
            "  \"commit_pairs\": 1,\n  \"transform_pairs\": 16,\n  \"steady_pairs\": 100,\n",
            "  \"limitations\": \"cold means renderer reset, not cold process; steady is finite workload, not thermal endurance; no queue/sample age or pen latency; record host OS/hardware/power separately\"\n}}\n"
        ),
        args[2],
        args[3],
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );
    let mut file = std::fs::File::options()
        .write(true)
        .create_new(true)
        .open(path.with_extension("json"))?;
    std::io::Write::write_all(&mut file, metadata.as_bytes())?;
    let mut fixture = Fixture::new();
    let mut measurement = Measurement::new(path)?;
    for (phase, count) in [("warmup", 5), ("cold", 10), ("warm", 32), ("after_idle", 4)] {
        for _ in 0..count {
            if phase == "cold" {
                measurement.clear_renderers();
            }
            if phase == "after_idle" {
                std::thread::sleep(Duration::from_millis(250));
            }
            let (scene, request, elapsed) = fixture.prepare();
            measurement.frame(phase, &scene, &request, elapsed, 0)?;
        }
    }
    fixture.command(EditorCommand::BeginStroke {
        pointer: PointerId(1),
        device: DeviceKind::Pen,
    });
    for id in 1..=32 {
        let input_ns = fixture.append(id);
        let (scene, request, elapsed) = fixture.prepare();
        measurement.frame("live", &scene, &request, elapsed, input_ns)?;
    }
    let input_ns = fixture.command(EditorCommand::CommitStroke(PointerId(1)));
    let (scene, request, elapsed) = fixture.prepare();
    measurement.frame("commit", &scene, &request, elapsed, input_ns)?;
    fixture.command(EditorCommand::SetTool(ToolId::Select));
    fixture.command(EditorCommand::SelectCycle(1));
    fixture.command(EditorCommand::BeginTransform);
    for step in 0..16 {
        let input_ns = fixture.command(EditorCommand::UpdateTransform {
            scale: 1.0 + step as f64 / 32.0,
            anchor_x: 100.0,
            anchor_y: 100.0,
        });
        let (scene, request, elapsed) = fixture.prepare();
        measurement.frame("transform", &scene, &request, elapsed, input_ns)?;
    }
    fixture.command(EditorCommand::CancelInteraction);
    for _ in 0..100 {
        let (scene, request, elapsed) = fixture.prepare();
        measurement.frame("steady", &scene, &request, elapsed, 0)?;
    }
    measurement.finish()
}
