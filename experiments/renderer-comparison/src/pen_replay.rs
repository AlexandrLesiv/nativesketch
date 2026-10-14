//! Experiment-only elapsed-time replay. Document edits remain in sketch-core.
use crate::{
    fixtures,
    native_fixture::{RcCommand, RcOutline},
};
use serde::{Deserialize, Serialize};
use sketch_core::{Brush, Editor, Input, Sample};
use std::{
    ffi::{CStr, CString, c_char, c_void},
    time::Instant,
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PcSnapshot {
    pub base_commands: *const RcCommand,
    pub base_command_count: u64,
    pub base_outlines: *const RcOutline,
    pub base_outline_count: u64,
    pub live_commands: *const RcCommand,
    pub live_command_count: u64,
    pub live_outlines: *const RcOutline,
    pub live_outline_count: u64,
    pub clip_commands: *const RcCommand,
    pub clip_command_count: u64,
    pub image_rgba: *const u8,
    pub image_byte_count: u64,
    pub document_revision: u64,
    pub live_revision: u64,
    pub viewport_revision: u64,
    pub last_sample_id: u64,
    pub total_samples: u64,
    pub delivered_now: u64,
    pub expected_samples: u64,
    pub last_sample_time_s: f64,
    pub first_delivered_time_s: f64,
    pub last_delivered_time_s: f64,
    pub input_ms: f64,
    pub outline_ms: f64,
    pub export_ms: f64,
    pub transform: [f64; 6],
    pub image_transform: [f64; 6],
    pub has_clip: u32,
    pub has_image: u32,
    pub trace_complete: u32,
    pub cache_allowed: u32,
    pub geometry_hash: [c_char; 65],
    pub document_hash: [c_char; 65],
}
#[derive(Clone, Deserialize, Serialize)]
struct Config {
    scene: String,
    trace: String,
    width: u32,
    height: u32,
    duration_s: f64,
    sample_hz: f64,
}
impl Config {
    fn validate(&self) -> Result<(), String> {
        if !["empty", "dense1000", "dense5000", "alpha", "clip", "image"]
            .contains(&self.scene.as_str())
            || !["short", "long", "panzoom"].contains(&self.trace.as_str())
            || !(64..=8192).contains(&self.width)
            || !(64..=8192).contains(&self.height)
            || !self.duration_s.is_finite()
            || !(0.0..=600.).contains(&self.duration_s)
            || self.duration_s == 0.
            || !self.sample_hz.is_finite()
            || !(30.0..=1000.).contains(&self.sample_hz)
            || (self.trace != "short"
                && self.duration_s * self.sample_hz >= sketch_core::MAX_SAMPLES as f64)
        {
            return Err("Invalid pen replay config or long-stroke sample limit".into());
        }
        Ok(())
    }
}
#[derive(Clone, Serialize)]
struct Event {
    time: f64,
    phase: u32,
    id: u64,
    x: f64,
    y: f64,
    pressure: f64,
}
fn trace(c: &Config) -> Vec<Event> {
    let mut events = vec![];
    let mut id = 0;
    let strokes = if c.trace == "short" {
        c.duration_s.ceil() as usize
    } else {
        1
    };
    for stroke in 0..strokes {
        let start = if c.trace == "short" {
            stroke as f64
        } else if c.trace == "panzoom" {
            c.duration_s * 0.5
        } else {
            0.
        };
        let end = if c.trace == "short" {
            (start + 0.75).min(c.duration_s)
        } else {
            c.duration_s
        };
        let steps = ((end - start) * c.sample_hz).floor() as usize;
        for n in 0..=steps {
            let time = (start + n as f64 / c.sample_hz).min(end);
            id += 1;
            let u = if c.trace == "short" {
                n as f64 / steps.max(1) as f64
            } else {
                (time * 0.5).fract()
            };
            events.push(Event {
                time,
                phase: if n == 0 { 0 } else { 1 },
                id,
                x: 180. + 1240. * u,
                y: 500. + 150. * (u * std::f64::consts::TAU + stroke as f64 * 0.3).sin(),
                pressure: 0.3 + 0.7 * (u * std::f64::consts::PI).sin().abs(),
            });
        }
        events.push(Event {
            time: end,
            phase: 2,
            id: 0,
            x: 0.,
            y: 0.,
            pressure: 0.,
        });
    }
    events
}
#[derive(Default)]
struct Geometry {
    commands: Vec<RcCommand>,
    outlines: Vec<RcOutline>,
    hash: String,
}
impl Geometry {
    fn new(commands: Vec<sketch_core::PathCommand>) -> Self {
        let paths = fixtures::group(commands);
        let mut result = Self {
            hash: fixtures::geometry_hash(&paths),
            ..Self::default()
        };
        for path in paths {
            result.outlines.push(RcOutline {
                stroke_id: path.id,
                command_start: result.commands.len() as u64,
                command_count: path.commands.len() as u64,
                rgba: path.rgba,
            });
            result
                .commands
                .extend(path.commands.into_iter().map(|c| RcCommand {
                    verb: c.verb,
                    reserved: 0,
                    points: c.points,
                }));
        }
        result
    }
}
struct Replay {
    config: Config,
    editor: Editor,
    events: Vec<Event>,
    index: usize,
    elapsed: f64,
    total: u64,
    expected_samples: u64,
    last_time: f64,
    document_revision: u64,
    base: Geometry,
    live: Geometry,
    clip: Vec<RcCommand>,
    image: Vec<u8>,
    document_hash: String,
    expected_document_hash: String,
    expected_geometry_hash: String,
    trace_hash: String,
    metadata: CString,
    failed: bool,
    controlled: bool,
}
impl Replay {
    fn new(config: Config, editor: Editor, image: Vec<u8>) -> Result<Self, String> {
        let events = trace(&config);
        let trace_hash = fixtures::hash(&serde_json::to_vec(&events).map_err(|e| e.to_string())?);
        let expected_samples = events.iter().filter(|e| e.phase <= 1).count() as u64;
        let document_hash = document_hash(&editor);
        let base = Geometry::new(editor.render_committed());
        // The same curve and rectangle as the historical fixture; both consumers clip identically.
        let clip = fixtures::clip_curve()
            .elements()
            .iter()
            .map(|e| {
                use vello_common::kurbo::PathEl;
                match *e {
                    PathEl::MoveTo(p) => RcCommand {
                        verb: 0,
                        reserved: 0,
                        points: [p.x, p.y, 0., 0., 0., 0.],
                    },
                    PathEl::LineTo(p) => RcCommand {
                        verb: 1,
                        reserved: 0,
                        points: [p.x, p.y, 0., 0., 0., 0.],
                    },
                    PathEl::CurveTo(a, b, c) => RcCommand {
                        verb: 2,
                        reserved: 0,
                        points: [a.x, a.y, b.x, b.y, c.x, c.y],
                    },
                    PathEl::ClosePath => RcCommand {
                        verb: 3,
                        reserved: 0,
                        points: [0.; 6],
                    },
                    PathEl::QuadTo(..) => unreachable!("fixture ellipse uses cubics"),
                }
            })
            .collect();
        Ok(Self {
            config,
            editor,
            events,
            index: 0,
            elapsed: 0.,
            total: 0,
            expected_samples,
            last_time: -1.,
            document_revision: 1,
            base,
            live: Geometry::new(vec![]),
            clip,
            image,
            document_hash,
            expected_document_hash: String::new(),
            expected_geometry_hash: String::new(),
            trace_hash,
            metadata: CString::new("").unwrap(),
            failed: false,
            controlled: false,
        })
    }
    fn advance(&mut self, elapsed: f64) -> Result<PcSnapshot, String> {
        if !elapsed.is_finite() || elapsed < self.elapsed || self.controlled {
            return Err("Replay time reversed/nonfinite or controls terminated replay".into());
        }
        self.elapsed = elapsed;
        let start = Instant::now();
        let before = self.total;
        let mut first = -1.;
        let mut committed = false;
        while let Some(e) = self.events.get(self.index).filter(|e| e.time <= elapsed) {
            if e.phase == 0 {
                self.editor
                    .begin(
                        1,
                        1,
                        Brush {
                            rgba: [0.12, 0.18, 0.72, 0.8],
                            width: 9.,
                        },
                    )
                    .map_err(|e| format!("{e:?}"))?;
            }
            if e.phase <= 1 {
                self.editor
                    .append(
                        1,
                        &[Input::Actual(Sample {
                            x: e.x,
                            y: e.y,
                            timestamp: e.time,
                            pressure: Some(e.pressure),
                            id: e.id,
                        })],
                    )
                    .map_err(|e| format!("{e:?}"))?;
                self.total += 1;
                self.last_time = e.time;
                if first < 0. {
                    first = e.time;
                }
            } else {
                self.editor.commit(1).map_err(|e| format!("{e:?}"))?;
                self.document_revision += 1;
                committed = true;
            }
            self.index += 1;
        }
        let input_ms = start.elapsed().as_secs_f64() * 1000.;
        let start = Instant::now();
        if committed {
            self.base = Geometry::new(self.editor.render_committed());
        }
        if self.total != before || committed {
            self.live = Geometry::new(self.editor.render_live());
        }
        let outline_ms = start.elapsed().as_secs_f64() * 1000.;
        let start = Instant::now();
        if committed {
            self.document_hash = document_hash(&self.editor);
        }
        let mut out = self.snapshot();
        out.input_ms = input_ms;
        out.outline_ms = outline_ms;
        out.delivered_now = self.total - before;
        out.first_delivered_time_s = first;
        out.last_delivered_time_s = if out.delivered_now > 0 {
            self.last_time
        } else {
            -1.
        };
        out.export_ms = start.elapsed().as_secs_f64() * 1000.;
        Ok(out)
    }
    fn snapshot(&self) -> PcSnapshot {
        let mut transform = [
            self.config.width as f64 / 1600.,
            0.,
            0.,
            self.config.height as f64 / 1000.,
            0.,
            0.,
        ];
        let pan_steps = (self.config.duration_s * 0.5 * self.config.sample_hz).floor() as u64;
        let pan_index = ((self.elapsed * self.config.sample_hz).floor() as u64).min(pan_steps);
        let moving = self.config.trace == "panzoom" && self.elapsed < self.config.duration_s * 0.5;
        if self.config.trace == "panzoom" {
            let t = pan_index as f64 / self.config.sample_hz;
            let zoom = 1. + 0.12 * (t * 1.1).sin();
            transform[0] *= zoom;
            transform[3] *= zoom;
            transform[4] = 30. * (t * 1.7).sin();
            transform[5] = 20. * (t * 1.3).sin();
        }
        let geometry_hash = fixtures::hash(
            format!("{}|{}|{:?}", self.base.hash, self.live.hash, transform).as_bytes(),
        );
        let mut out = PcSnapshot {
            base_commands: self.base.commands.as_ptr(),
            base_command_count: self.base.commands.len() as u64,
            base_outlines: self.base.outlines.as_ptr(),
            base_outline_count: self.base.outlines.len() as u64,
            live_commands: self.live.commands.as_ptr(),
            live_command_count: self.live.commands.len() as u64,
            live_outlines: self.live.outlines.as_ptr(),
            live_outline_count: self.live.outlines.len() as u64,
            clip_commands: self.clip.as_ptr(),
            clip_command_count: self.clip.len() as u64,
            image_rgba: self.image.as_ptr(),
            image_byte_count: self.image.len() as u64,
            document_revision: self.document_revision,
            live_revision: self.total * 2 + u64::from(self.editor.state().interaction != 0),
            viewport_revision: if self.config.trace == "panzoom" {
                pan_index + 1
            } else {
                1
            },
            last_sample_id: self.total,
            total_samples: self.total,
            delivered_now: 0,
            expected_samples: self.expected_samples,
            last_sample_time_s: self.last_time,
            first_delivered_time_s: -1.,
            last_delivered_time_s: -1.,
            input_ms: 0.,
            outline_ms: 0.,
            export_ms: 0.,
            transform,
            image_transform: fixtures::image_transform(0),
            has_clip: u32::from(self.config.scene == "clip"),
            has_image: u32::from(self.config.scene == "image"),
            trace_complete: u32::from(self.index == self.events.len()),
            cache_allowed: u32::from(!moving),
            geometry_hash: [0; 65],
            document_hash: [0; 65],
        };
        copy_hash(&mut out.geometry_hash, &geometry_hash);
        copy_hash(&mut out.document_hash, &self.document_hash);
        out
    }
    fn control(&mut self, action: u32) -> Result<PcSnapshot, String> {
        match action {
            0 => self.editor.cancel(),
            1 => self.editor.undo().map_err(|e| format!("{e:?}"))?,
            2 => self.editor.redo().map_err(|e| format!("{e:?}"))?,
            _ => return Err("Unknown control".into()),
        }
        self.controlled = true;
        self.document_revision += 1;
        self.base = Geometry::new(self.editor.render_committed());
        self.live = Geometry::new(self.editor.render_live());
        self.document_hash = document_hash(&self.editor);
        Ok(self.snapshot())
    }
}
fn document_hash(editor: &Editor) -> String {
    let mut bytes = Vec::new();
    for stroke in &editor.document().strokes {
        bytes.extend_from_slice(&stroke.id.to_le_bytes());
        bytes.extend_from_slice(&stroke.device.to_le_bytes());
        bytes.extend_from_slice(&stroke.pointer_id.to_le_bytes());
        for v in stroke.brush.rgba.into_iter().chain([
            stroke.brush.width,
            stroke.transform.scale,
            stroke.transform.tx,
            stroke.transform.ty,
        ]) {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        bytes.extend_from_slice(&(stroke.samples.len() as u64).to_le_bytes());
        for sample in &stroke.samples {
            bytes.extend_from_slice(&sample.id.to_le_bytes());
            for v in [sample.x, sample.y, sample.timestamp] {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            bytes.push(u8::from(sample.pressure.is_some()));
            if let Some(p) = sample.pressure {
                bytes.extend_from_slice(&p.to_le_bytes());
            }
        }
    }
    fixtures::hash(&bytes)
}
fn copy_hash(out: &mut [c_char; 65], s: &str) {
    for (a, b) in out.iter_mut().zip(s.bytes()) {
        *a = b as c_char;
    }
}
fn hash_string(s: &[c_char; 65]) -> String {
    s.iter()
        .take_while(|v| **v != 0)
        .map(|v| *v as u8 as char)
        .collect()
}
fn call(h: *mut c_void, f: impl FnOnce(&mut Replay) -> Result<(), String>) -> i32 {
    let result = crate::ffi(|| {
        let r = unsafe { h.cast::<Replay>().as_mut() }.ok_or("Null replay")?;
        if r.failed {
            return Err("Replay poisoned; destroy".into());
        }
        f(r)
    });
    if result != 0
        && let Some(r) = unsafe { h.cast::<Replay>().as_mut() }
    {
        r.failed = true;
    }
    result
}
/// # Safety
/// config points to a NUL-terminated UTF-8 string; see pen_comparison.h.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pc_replay_create(config: *const c_char) -> *mut c_void {
    let mut result = std::ptr::null_mut();
    crate::ffi(|| {
        if config.is_null() {
            return Err("Null replay config".into());
        }
        let c: Config = serde_json::from_str(
            unsafe { CStr::from_ptr(config) }
                .to_str()
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        c.validate()?;
        let (editor, image) = if c.scene == "empty" {
            (Editor::new(), vec![])
        } else {
            let f = fixtures::Fixture::new(&c.scene)?;
            (f.editor, if c.scene == "image" { f.image } else { vec![] })
        };
        let bytes = editor.export().map_err(|e| format!("{e:?}"))?.0;
        let mut expected_editor = Editor::new();
        expected_editor
            .import(&bytes)
            .map_err(|e| format!("{e:?}"))?;
        let mut expected = Replay::new(c.clone(), expected_editor, vec![])?;
        let final_frame = expected.advance(c.duration_s)?;
        let mut r = Replay::new(c, editor, image)?;
        r.expected_document_hash = hash_string(&final_frame.document_hash);
        r.expected_geometry_hash = hash_string(&final_frame.geometry_hash);
        result = Box::into_raw(Box::new(r)).cast();
        Ok(())
    });
    result
}
/// # Safety
/// Handle and writable output satisfy pen_comparison.h; no concurrent use.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pc_replay_advance(
    h: *mut c_void,
    elapsed: f64,
    out: *mut PcSnapshot,
) -> i32 {
    call(h, |r| {
        if out.is_null() {
            return Err("Null snapshot".into());
        }
        unsafe { *out = r.advance(elapsed)? };
        Ok(())
    })
}
/// # Safety
/// Handle and writable output satisfy pen_comparison.h; no concurrent use.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pc_replay_control(
    h: *mut c_void,
    action: u32,
    out: *mut PcSnapshot,
) -> i32 {
    call(h, |r| {
        if out.is_null() {
            return Err("Null snapshot".into());
        }
        unsafe { *out = r.control(action)? };
        Ok(())
    })
}
/// # Safety
/// Handle satisfies pen_comparison.h; returned string is borrowed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pc_replay_metadata_json(h: *mut c_void) -> *const c_char {
    let mut result = std::ptr::null();
    call(h, |r| {
        r.metadata=CString::new(serde_json::json!({
        "trace_samples":r.events.iter().filter(|e|e.phase<=1).map(|e|serde_json::json!({"id":e.id,"time_s":e.time,"phase":e.phase,"x":e.x,"y":e.y,"pressure":e.pressure})).collect::<Vec<_>>(),
        "image_sha256":fixtures::hash(&r.image),
        "clip_intent":"intersection of document rect(150,130)-(1450,870) and fixed ellipse path; static and live equally clipped",
        "abi":1,"input_kind":"synthetic elapsed-time pressure trace; NOT recorded Wacom input",
        "config":r.config,"trace_sha256":r.trace_hash,"expected_samples":r.events.iter().filter(|e|e.phase<=1).count(),
        "expected_final_document_hash":r.expected_document_hash,"expected_final_geometry_hash":r.expected_geometry_hash,
        "geometry_hash_definition":"SHA256(base geometry digest | live geometry digest | exact transform)",
        "document_hash_definition":"SHA256 canonical ordered document fields and real samples; no predicted data",
        "initial_document_revision":1,"clock":"relative monotonic seconds supplied by host",
        "sample_preservation":"all due actual samples applied in original order; no prediction",
        "cache_handoff":"synchronous painter rebuild; new base and empty live published atomically",
        "cache_invalidation":"document_revision + viewport_revision + fixed size/color",
        "hash_work":"geometry grouping/hash included in outline_ms; canonical document field hash in export_ms on commits",
        "source_revision":option_env!("RENDERER_SOURCE_REVISION").unwrap_or("unrecorded")
    }).to_string()).map_err(|e|e.to_string())?;
        result = r.metadata.as_ptr();
        Ok(())
    });
    result
}
/// # Safety
/// Handle null or uniquely owned valid replay, destroyed only once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pc_replay_destroy(h: *mut c_void) {
    if !h.is_null() {
        crate::ffi(|| {
            drop(unsafe { Box::from_raw(h.cast::<Replay>()) });
            Ok(())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn pc_error() -> *const c_char {
    crate::rc_error(std::ptr::null_mut())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn replay(trace: &str) -> Replay {
        Replay::new(
            Config {
                scene: "empty".into(),
                trace: trace.into(),
                width: 1600,
                height: 1000,
                duration_s: 3.,
                sample_hz: 120.,
            },
            Editor::new(),
            vec![],
        )
        .unwrap()
    }
    #[test]
    fn catchup_preserves_samples_and_final_geometry() {
        for trace in ["short", "long", "panzoom"] {
            let mut fast = replay(trace);
            let mut slow = replay(trace);
            for i in 0..=600 {
                fast.advance(i as f64 / 200.).unwrap();
            }
            let a = fast.snapshot();
            let b = slow.advance(3.).unwrap();
            assert_eq!(a.total_samples, b.total_samples);
            assert_eq!(a.document_hash, b.document_hash);
            assert_eq!(a.geometry_hash, b.geometry_hash);
            assert_eq!(a.total_samples, a.expected_samples);
            assert_eq!(a.trace_complete, 1);
            assert_eq!(a.live_outline_count, 0);
            assert_eq!(
                fast.editor
                    .document()
                    .strokes
                    .iter()
                    .map(|s| s.samples.len())
                    .sum::<usize>() as u64,
                a.total_samples
            );
        }
    }
    #[test]
    fn cache_revisions_only_change_on_document_or_view_changes() {
        let mut r = replay("short");
        let a = r.advance(0.).unwrap();
        let b = r.advance(0.2).unwrap();
        assert_eq!(a.document_revision, b.document_revision);
        assert_ne!(a.live_revision, b.live_revision);
        assert_eq!(a.base_commands, b.base_commands);
        let c = r.advance(0.75).unwrap();
        assert_eq!(c.document_revision, b.document_revision + 1);
        assert_eq!(c.live_outline_count, 0);
        assert_eq!(c.base_outline_count, 1);
    }
    #[test]
    fn cancellation_and_undo_redo_preserve_committed_state() {
        let mut r = replay("short");
        r.advance(1.2).unwrap();
        let committed = r.editor.document().clone();
        r.control(0).unwrap();
        assert_eq!(r.editor.document(), &committed);
        assert!(r.live.outlines.is_empty());
        assert!(r.advance(2.).is_err());
        let hash = r.document_hash.clone();
        r.control(1).unwrap();
        assert!(r.editor.document().strokes.is_empty());
        r.control(2).unwrap();
        assert_eq!(r.document_hash, hash);
    }
    #[test]
    fn rejects_time_reversal_and_bad_config() {
        let mut r = replay("long");
        r.advance(1.).unwrap();
        assert!(r.advance(0.).is_err());
        assert!(r.advance(f64::NAN).is_err());
        let mut c = r.config;
        c.duration_s = -1.;
        assert!(c.validate().is_err());
        c.duration_s = 600.;
        c.sample_hz = 1000.;
        assert!(c.validate().is_err());
    }
    #[test]
    fn panzoom_disables_cache_until_navigation_finishes() {
        let mut r = replay("panzoom");
        let a = r.advance(0.5).unwrap();
        assert_eq!(a.total_samples, 0);
        assert_eq!(a.cache_allowed, 0);
        let b = r.advance(1.5).unwrap();
        assert_eq!(b.total_samples, 1);
        assert_eq!(b.cache_allowed, 1);
        let c = r.advance(2.0).unwrap();
        assert_eq!(b.viewport_revision, c.viewport_revision);
        assert_eq!(b.transform, c.transform);
    }
    #[test]
    fn seeded_document_oracle_matches_chunked_delivery() {
        let f = fixtures::Fixture::new("dense1000").unwrap();
        let bytes = f.editor.export().unwrap().0;
        let mut imported = Editor::new();
        imported.import(&bytes).unwrap();
        let mut c = replay("short").config;
        c.scene = "dense1000".into();
        c.duration_s = 1.2;
        let mut a = Replay::new(c.clone(), f.editor, vec![]).unwrap();
        let mut b = Replay::new(c, imported, vec![]).unwrap();
        // A single catch-up can commit one stroke AND begin another before hashing.
        let chunked = a.advance(1.1).unwrap();
        assert_eq!(chunked.base_outline_count, 1001);
        assert_eq!(chunked.live_outline_count, 1);
        let a = a.advance(1.2).unwrap();
        let b = b.advance(1.2).unwrap();
        assert_eq!(a.document_hash, b.document_hash);
        assert_eq!(a.geometry_hash, b.geometry_hash);
        assert_eq!(a.total_samples, a.expected_samples);
    }
    #[test]
    fn ffi_expected_identity_and_poisoning() {
        unsafe {
            let config=CString::new(r#"{"scene":"empty","trace":"short","width":1600,"height":1000,"duration_s":2.0,"sample_hz":120.0}"#).unwrap();
            let h = pc_replay_create(config.as_ptr());
            assert!(!h.is_null());
            let meta: serde_json::Value =
                serde_json::from_str(CStr::from_ptr(pc_replay_metadata_json(h)).to_str().unwrap())
                    .unwrap();
            let mut out = std::mem::MaybeUninit::uninit();
            assert_eq!(pc_replay_advance(h, 2., out.as_mut_ptr()), 0);
            let out = out.assume_init();
            assert_eq!(
                hash_string(&out.document_hash),
                meta["expected_final_document_hash"]
            );
            assert_eq!(
                hash_string(&out.geometry_hash),
                meta["expected_final_geometry_hash"]
            );
            assert_eq!(pc_replay_advance(h, 3., std::ptr::null_mut()), 1);
            assert!(pc_replay_metadata_json(h).is_null());
            pc_replay_destroy(h);
            assert!(pc_replay_create(std::ptr::null()).is_null());
        }
    }
}
