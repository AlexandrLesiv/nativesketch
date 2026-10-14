//! Deterministic renderer-independent inputs. No renderer-specific stroke tessellation.
use sha2::{Digest, Sha256};
use sketch_core::{Brush, Editor, Input, PathCommand, Sample};
use std::time::Instant;

pub const IDS: &[&str] = &[
    "live",
    "dense100",
    "dense1000",
    "dense5000",
    "panzoom",
    "alpha",
    "clip",
    "image",
];
pub const SPEC: &str = "native-sketch-renderer-fixtures-v1:seed=0;document=1600x1000;nonzero;source-over;cliprect=150,130,1450,870;clipellipse=800,500,570,310,tolerance0.05";
#[derive(Clone)]
pub struct Outline {
    pub id: u64,
    pub rgba: [f64; 4],
    pub commands: Vec<PathCommand>,
}
pub fn group(commands: Vec<PathCommand>) -> Vec<Outline> {
    let mut result: Vec<Outline> = Vec::new();
    for c in commands {
        if result.last().is_none_or(|p| p.id != c.stroke_id) {
            result.push(Outline {
                id: c.stroke_id,
                rgba: c.rgba,
                commands: Vec::new(),
            });
        }
        result.last_mut().unwrap().commands.push(c);
    }
    result
}
pub fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn geometry_hash(paths: &[Outline]) -> String {
    let mut h = Sha256::new();
    for p in paths {
        h.update(p.id.to_le_bytes());
        for v in p.rgba {
            h.update(v.to_le_bytes());
        }
        for c in &p.commands {
            h.update(c.verb.to_le_bytes());
            for v in c.points {
                h.update(v.to_le_bytes());
            }
        }
    }
    format!("{:x}", h.finalize())
}
pub struct Fixture {
    pub id: String,
    pub editor: Editor,
    pub paths: Vec<Outline>,
    pub geometry_hash: String,
    pub spec_hash: String,
    pub input_hash: String,
    pub image_hash: String,
    pub image: Vec<u8>,
    pub input_ms: f64,
    pub outline_ms: f64,
    pub identity_precompute_ms: f64,
    live_geometry_hashes: Vec<String>,
    live_state_hashes: Vec<String>,
    live_state_dimensions: (u32, u32),
    last: Option<usize>,
    initialized: bool,
}
impl Fixture {
    pub fn new(id: &str) -> Result<Self, String> {
        if !IDS.contains(&id) {
            return Err(format!("Unknown fixture: {id}"));
        }
        let count = match id {
            "dense100" | "live" => 100,
            "dense1000" | "panzoom" => 1000,
            "dense5000" => 5000,
            "alpha" => 256,
            "clip" => 160,
            _ => 20,
        };
        let mut editor = Editor::default();
        let columns = (f64::sqrt(count as f64 * 1.6).ceil()) as usize;
        for n in 0..count {
            let alpha = if id == "alpha" {
                [0.15, 0.35, 0.6, 0.85][n % 4]
            } else {
                1.0
            };
            let rgba = [
                ((n * 37) % 200) as f64 / 255.0,
                ((n * 67 + 30) % 210) as f64 / 255.0,
                ((n * 17 + 40) % 220) as f64 / 255.0,
                alpha,
            ];
            editor
                .begin(
                    1,
                    1,
                    Brush {
                        rgba,
                        width: if id == "alpha" { 30.0 } else { 7.0 },
                    },
                )
                .map_err(|e| format!("{e:?}"))?;
            let inputs: Vec<_> = (0..24)
                .map(|j| {
                    let t = j as f64 / 23.0;
                    let (x, y) = if id == "alpha" {
                        (
                            300.0 + t * 1000.0,
                            500.0 + (t * 8.0 + n as f64 * 0.13).sin() * 220.0,
                        )
                    } else {
                        (
                            40.0 + (n % columns) as f64 * 1450.0 / columns as f64 + t * 70.0,
                            40.0 + (n / columns) as f64 * 880.0 / (count.div_ceil(columns)) as f64
                                + (t * 6.0 + n as f64).sin() * 12.0,
                        )
                    };
                    Input::Actual(Sample {
                        x,
                        y,
                        timestamp: j as f64 / 120.0,
                        pressure: Some(0.3 + 0.7 * t),
                        id: j as u64 + 1,
                    })
                })
                .collect();
            editor.append(1, &inputs).map_err(|e| format!("{e:?}"))?;
            editor.commit(1).map_err(|e| format!("{e:?}"))?;
        }
        let mut reader = png::Decoder::new(std::io::Cursor::new(include_bytes!(
            "../fixtures/image.png"
        )))
        .read_info()
        .map_err(|e| e.to_string())?;
        let mut image = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut image).map_err(|e| e.to_string())?;
        image.truncate(info.buffer_size());
        if info.width != 2048 || info.height != 2048 || info.color_type != png::ColorType::Rgba {
            return Err("Unexpected image fixture format".into());
        }
        let image_hash = hash(&image);
        let spec_hash = hash(format!("{SPEC};id={id};count={count};samples=24").as_bytes());
        let mut trace = Vec::new();
        for f in 0..600 {
            for v in transform(id, f, 1600, 1000) {
                trace.extend_from_slice(&v.to_le_bytes());
            }
            if id == "image" {
                for v in image_transform(f) {
                    trace.extend_from_slice(&v.to_le_bytes());
                }
            }
        }
        for j in 0..240 {
            let p = live_sample(j);
            for v in [p.x, p.y, p.timestamp, p.pressure.unwrap()] {
                trace.extend_from_slice(&v.to_le_bytes());
            }
        }
        let mut fixture = Self {
            id: id.into(),
            editor,
            paths: vec![],
            geometry_hash: String::new(),
            spec_hash,
            input_hash: hash(&trace),
            image_hash,
            image,
            input_ms: 0.0,
            outline_ms: 0.0,
            identity_precompute_ms: 0.0,
            live_geometry_hashes: Vec::new(),
            live_state_hashes: Vec::new(),
            live_state_dimensions: (0, 0),
            last: None,
            initialized: false,
        };
        if id == "live" {
            let start = Instant::now();
            // Run the actual core input/outline path once, before GPU initialization, to
            // validate the expected cycle. Never hash the complete live scene in a timed loop.
            for frame in 0..240 {
                fixture.prepare(frame)?;
                fixture
                    .live_geometry_hashes
                    .push(geometry_hash(&fixture.paths));
            }
            fixture.configure_identity(1600, 1000);
            fixture.editor.cancel();
            fixture.paths.clear();
            fixture.geometry_hash.clear();
            fixture.last = None;
            fixture.initialized = false;
            fixture.input_ms = 0.0;
            fixture.outline_ms = 0.0;
            fixture.identity_precompute_ms = start.elapsed().as_secs_f64() * 1000.0;
        }
        Ok(fixture)
    }
    pub fn configure_identity(&mut self, width: u32, height: u32) {
        if self.id == "live" {
            self.live_state_hashes = self
                .live_geometry_hashes
                .iter()
                .enumerate()
                .map(|(frame, geometry)| {
                    compose_state_hash(&self.id, geometry, frame, width, height)
                })
                .collect();
            self.live_state_dimensions = (width, height);
        }
    }
    pub fn expected_live_sequence_hash(&self) -> Option<String> {
        (!self.live_state_hashes.is_empty())
            .then(|| hash(self.live_state_hashes.concat().as_bytes()))
    }
    pub fn prepare(&mut self, frame: usize) -> Result<bool, String> {
        self.input_ms = 0.0;
        self.outline_ms = 0.0;
        let changed = !self.initialized || self.id == "live";
        if self.id == "live" {
            let t = Instant::now();
            let index = frame % 240;
            if self.last.is_none_or(|old| index <= old) {
                self.editor.cancel();
                self.editor
                    .begin(
                        9,
                        1,
                        Brush {
                            rgba: [0.15, 0.2, 0.7, 0.65],
                            width: 22.0,
                        },
                    )
                    .map_err(|e| format!("{e:?}"))?;
                self.last = None;
            }
            let start = self.last.map_or(0, |x| x + 1);
            let data: Vec<_> = (start..=index)
                .map(|j| Input::Actual(live_sample(j)))
                .collect();
            self.editor.append(9, &data).map_err(|e| format!("{e:?}"))?;
            self.last = Some(index);
            self.input_ms = t.elapsed().as_secs_f64() * 1000.0;
        }
        if changed {
            let t = Instant::now();
            self.paths = group(self.editor.render());
            self.outline_ms = t.elapsed().as_secs_f64() * 1000.0;
            self.geometry_hash.clear();
            self.initialized = true;
        }
        Ok(changed)
    }
    pub fn state_hash(&mut self, frame: usize, width: u32, height: u32) -> String {
        if self.id == "live" {
            if self.live_state_dimensions != (width, height) {
                self.configure_identity(width, height);
            }
            return self.live_state_hashes[frame % 240].clone();
        }
        if self.geometry_hash.is_empty() {
            self.geometry_hash = geometry_hash(&self.paths);
        }
        compose_state_hash(&self.id, &self.geometry_hash, frame, width, height)
    }
    /// Untimed correctness guard. Recomputes the actual current geometry, never the cache.
    pub fn actual_state_hash(&self, frame: usize, width: u32, height: u32) -> String {
        compose_state_hash(&self.id, &geometry_hash(&self.paths), frame, width, height)
    }
}
fn compose_state_hash(id: &str, geometry: &str, frame: usize, width: u32, height: u32) -> String {
    let mut bytes = geometry.as_bytes().to_vec();
    for v in transform(id, frame, width, height) {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    if id == "image" {
        for v in image_transform(frame) {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
    }
    hash(&bytes)
}
fn live_sample(j: usize) -> Sample {
    let t = j as f64 / 239.0;
    Sample {
        x: 100.0 + t * 1400.0,
        y: 500.0 + (t * 12.0).sin() * 230.0,
        timestamp: j as f64 / 120.0,
        pressure: Some(0.15 + 0.85 * (t * std::f64::consts::PI).sin()),
        id: j as u64 + 1,
    }
}
pub fn transform(id: &str, frame: usize, width: u32, height: u32) -> [f64; 6] {
    let t = (frame % 600) as f64 / 600.0 * std::f64::consts::TAU;
    let scale = if id == "panzoom" {
        1.0 + 0.18 * t.sin()
    } else {
        1.0
    };
    let tx = if id == "panzoom" {
        100.0 * t.cos()
    } else {
        0.0
    };
    let ty = if id == "panzoom" { 70.0 * t.sin() } else { 0.0 };
    let sx = width as f64 / 1600.0;
    let sy = height as f64 / 1000.0;
    [sx * scale, 0.0, 0.0, sy * scale, sx * tx, sy * ty]
}
pub fn image_transform(frame: usize) -> [f64; 6] {
    let s = (0.85 + 0.1 * ((frame % 600) as f64 / 600.0 * std::f64::consts::TAU).sin()) * 900.0
        / 2048.0;
    [s, 0.0, 0.0, s, 350.0, 50.0]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grouped_translucent_stroke_remains_one_fill() {
        let mut f = Fixture::new("alpha").unwrap();
        f.prepare(0).unwrap();
        assert_eq!(f.paths.len(), 256);
        assert!(f.paths[0].commands.iter().filter(|c| c.verb == 0).count() > 24);
        assert!(f.paths.iter().all(|p| p.rgba[3] < 1.0));
    }
    #[test]
    fn traces_repeat_without_committing_live_strokes() {
        let mut a = Fixture::new("live").unwrap();
        a.prepare(0).unwrap();
        let first = a.state_hash(0, 1600, 1000);
        assert_eq!(first, a.actual_state_hash(0, 1600, 1000));
        for i in 1..240 {
            a.prepare(i).unwrap();
            assert_eq!(
                a.state_hash(i, 1600, 1000),
                a.actual_state_hash(i, 1600, 1000),
                "cycle frame {i}"
            );
        }
        a.prepare(240).unwrap();
        assert_eq!(a.state_hash(240, 1600, 1000), first);
        assert_eq!(a.actual_state_hash(240, 1600, 1000), first);
        // Warm-up ends at 119; measured phase restarts at zero on warmed resources.
        a.prepare(119).unwrap();
        a.prepare(0).unwrap();
        assert_eq!(a.actual_state_hash(0, 1600, 1000), first);
        assert_eq!(a.state_hash(0, 1600, 1000), first);
        a.configure_identity(2560, 1600);
        assert_eq!(
            a.state_hash(0, 2560, 1600),
            a.actual_state_hash(0, 2560, 1600)
        );
        assert_eq!(a.editor.state().count, 100);
    }
    #[test]
    fn fixtures_are_reproducible_and_distinct() {
        let mut a = Fixture::new("clip").unwrap();
        let mut b = Fixture::new("clip").unwrap();
        a.prepare(0).unwrap();
        b.prepare(0).unwrap();
        assert_eq!(a.state_hash(0, 1600, 1000), b.state_hash(0, 1600, 1000));
        assert_ne!(a.spec_hash, Fixture::new("image").unwrap().spec_hash);
        assert!(Fixture::new("missing").is_err());
    }
}
