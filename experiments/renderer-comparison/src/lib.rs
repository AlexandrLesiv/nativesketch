//! Standalone renderer experiment; no production ABI or backend commitment.
mod fixtures;
mod gpu;
mod native_fixture;
use objc2_metal::{MTLCommandBuffer, MTLCommandBufferStatus, MTLDevice};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    collections::VecDeque,
    ffi::{CStr, CString, c_char, c_void},
    fs::{self, File},
    io::{BufWriter, Write},
    path::PathBuf,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Config {
    backend: String,
    fixture: String,
    width: u32,
    height: u32,
    mode: String,
    warmup: u64,
    #[serde(default)]
    frames_requested: Option<u64>,
    #[serde(default)]
    duration_seconds: Option<f64>,
    repeat_id: String,
    output: PathBuf,
    #[serde(default)]
    display: Value,
    #[serde(default)]
    runtime: Value,
    #[serde(default)]
    power: Value,
    #[serde(default)]
    build: Value,
}
impl Config {
    fn validate(&self) -> Result<(), String> {
        if !["skia", "vello"].contains(&self.backend.as_str())
            || !fixtures::IDS.contains(&self.fixture.as_str())
            || !["completion", "paced"].contains(&self.mode.as_str())
        {
            return Err("Invalid backend, fixture or mode".into());
        }
        if !(64..=8192).contains(&self.width) || !(64..=8192).contains(&self.height) {
            return Err("Dimensions must be between 64 and 8192".into());
        }
        if self.warmup > 100000
            || self.frames_requested.is_some_and(|f| f == 0 || f > 1000000)
            || self
                .duration_seconds
                .is_some_and(|s| !s.is_finite() || s <= 0.0 || s > 14400.0)
            || self.frames_requested.is_some() == self.duration_seconds.is_some()
        {
            return Err("Specify positive frames_requested OR duration_seconds".into());
        }
        if !self.output.is_absolute() || self.repeat_id.is_empty() {
            return Err("Absolute output and nonempty repeat_id required".into());
        }
        Ok(())
    }
}
#[derive(Default)]
struct Frame {
    phase: &'static str,
    index: u64,
    state_hash: String,
    input_ms: f64,
    outline_ms: f64,
    scene_ms: f64,
    acquire_ms: f64,
    render_submit_ms: f64,
    present_enqueue_ms: f64,
    completion_wait_ms: f64,
    service_ms: f64,
    cpu_frame_start_s: f64,
    cpu_submit_s: f64,
    gpu_start_s: Option<f64>,
    gpu_end_s: Option<f64>,
    gpu_span_ms: Option<f64>,
    presented_s: Option<f64>,
    ticket: Option<gpu::Ticket>,
    present_signal: Option<std::sync::Arc<std::sync::atomic::AtomicU64>>,
}
struct Session {
    backend: gpu::Backend,
    gpu: gpu::Gpu,
    fixture: fixtures::Fixture,
    config: Config,
    frames: Vec<Frame>,
    pending: VecDeque<usize>,
    manifest: Value,
    finished: bool,
    last_phase: Option<u32>,
    next_index: u64,
}
fn now() -> f64 {
    objc2_quartz_core::CACurrentMediaTime()
}
fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}
fn write_json(path: PathBuf, value: &Value) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
    fs::rename(tmp, path).map_err(|e| e.to_string())
}
fn log_event(output: &std::path::Path, message: &str) -> Result<(), String> {
    let mut log = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(output.join("run.log"))
        .map_err(|e| e.to_string())?;
    writeln!(log, "{:.9} {}", now(), message).map_err(|e| e.to_string())
}
impl Session {
    fn record_failure(&mut self, stage: &str, message: &str) {
        self.manifest["status"] = json!("failed");
        self.manifest["error"] = json!(message);
        self.manifest["error_stage"] = json!(stage);
        self.manifest["completed_sample_count"] =
            json!(self.frames.iter().filter(|f| f.phase == "measured").count());
        let _ = log_event(
            &self.config.output,
            &format!(
                "ERROR stage={stage} frames={} cause={message}",
                self.frames.len()
            ),
        );
        let _ = self.write_frames();
        let _ = write_json(self.config.output.join("manifest.json"), &self.manifest);
    }
    unsafe fn new(config: Config, layer: *mut c_void) -> Result<Self, String> {
        config.validate()?;
        if config.output.join("manifest.json").exists() {
            return Err("Output already contains a manifest; choose a fresh directory".into());
        }
        fs::create_dir_all(&config.output).map_err(|e| e.to_string())?;
        let init = Instant::now();
        let mut manifest = serde_json::to_value(&config).map_err(|e| e.to_string())?;
        manifest["schema_version"] = json!(1);
        manifest["status"] = json!("running");
        manifest["source_revision"] =
            json!(option_env!("RENDERER_SOURCE_REVISION").unwrap_or("unrecorded"));
        manifest["source_diff_hash"] =
            json!(option_env!("RENDERER_SOURCE_DIFF_HASH").unwrap_or("unrecorded"));
        manifest["build_embedded"] =
            serde_json::from_str(option_env!("RENDERER_BUILD_JSON").unwrap_or("{} "))
                .unwrap_or(json!({}));
        manifest["dependencies"] = json!({"skia-safe":"0.153.3","vello_gpu":"0.3.0","vello_common":"0.3.0","wgpu":"30.0.0","lockfile_sha256":fixtures::hash(include_bytes!("../Cargo.lock")),"skia_features":["ganesh","metal","binary-cache","jpeg","pdf"],"skia_upstream_ref":"m153-0.101.2","skia_binary_url":"https://github.com/rust-skia/skia-binaries/releases/download/0.153.3/skia-binaries-b7f043e0b1e2a850e702-aarch64-apple-darwin-ganesh-jpegd-jpege-metal-pdf.tar.gz","skia_binary_release_digest":"sha256:59ac182e5b81445ba8bb84c5a65468e3aec7b6ee1b642868603a3725f29a1059","skia_packaging_note":"Minimal ganesh-metal binary unavailable (404); published jpeg/pdf feature package used. JPEG/PDF APIs not exercised."});
        manifest["backend_variant"] = json!(if config.backend == "skia" {
            "Ganesh Metal"
        } else {
            "Sparse Strips CPU preprocessing / wgpu Metal GPU rasterization and compositing"
        });
        manifest["quality"] = json!({"target":"BGRA8Unorm","alpha":"premultiplied target / unpremultiplied source RGBA","color_space":"sRGB numeric channels; no linear-light conversion","blend":"source-over","fill":"nonzero once per complete stroke","image_filter":"bilinear no mipmaps","antialiasing":if config.backend=="skia"{"Ganesh analytic coverage AA; sample_count=1"}else{"Sparse Strips analytic coverage; sample_count=1"}});
        manifest["timing_provenance"] = json!({"cpu_clock":"CACurrentMediaTime monotonic seconds; durations Instant","cpu_frame_start_s":"entry BEFORE retirement wait; successive starts include backpressure/host scheduling/hash overhead","service_ms":"after prior-frame retirement, before input through common presentation enqueue; excludes hash/readback/output; includes acquisition","gpu_span_ms":"Metal marker before backend scene preparation GPUStartTime through shared blit/present command GPUEndTime; includes all backend CPU preprocessing/encoding gaps and queue scheduling; excludes common input/outline and drawable acquisition; NOT pure GPU execution","presented_s":"MTLDrawable.addPresentedHandler presentedTime (seconds); callback positive-only","completion_wait_ms":"retirement wait before service plus completion-mode current-frame wait","image_upload":"first-use scene preparation/queue staging and render submission; no eager GPU upload","state_hash":"live: cached 240-frame expected core geometry/state identities, precomputed before GPU initialization; actual input/outline work still executed each frame; final actual identity recomputed outside timing. Other fixtures cache immutable geometry and hash only the changing transforms after presentation enqueue.","inflight":if config.mode=="paced"{2}else{1},"presentation":"same Metal queue, tracked BGRA output, full texture blit, CAMetalLayer displaySync mode dependent"});
        manifest["unsupported"] = json!({"gpu_execution_ms":"No equivalent pure-pass timing; common timeline span only","physical_pen_latency":"No external measurement equipment","presentation_deadline_misses":"Callback intervals available; no independent display deadline schedule","gpu_peak_memory":"No comparable peak resident GPU counter","physical_footprint":"Not implemented; OS peak RSS and external sampler separate"});
        write_json(config.output.join("manifest.json"), &manifest)?;
        log_event(
            &config.output,
            &format!(
                "INIT source={} diff={} config={}",
                manifest["source_revision"],
                manifest["source_diff_hash"],
                serde_json::to_string(&config).map_err(|e| e.to_string())?
            ),
        )?;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut fixture = fixtures::Fixture::new(&config.fixture)?;
            fixture.configure_identity(config.width, config.height);
            let gpu = unsafe {
                gpu::Gpu::new(
                    layer.cast(),
                    config.width,
                    config.height,
                    config.mode == "paced",
                )
            }?;
            let backend = gpu::Backend::new(&config.backend, &gpu)?;
            Ok::<_, String>((fixture, gpu, backend))
        }))
        .map_err(|_| "Rust panic during renderer initialization".to_string())
        .and_then(|r| r);
        let (fixture, gpu, backend) = match result {
            Ok(v) => v,
            Err(e) => {
                manifest["status"] = json!("failed");
                manifest["error"] = json!(e);
                let _ = log_event(&config.output, &format!("ERROR stage=init cause={e}"));
                write_json(config.output.join("manifest.json"), &manifest)?;
                return Err(e);
            }
        };
        log_event(
            &config.output,
            &format!(
                "READY backend={} gpu={} fixture={} init_ms={:.6}",
                config.backend,
                gpu.name,
                config.fixture,
                ms(init)
            ),
        )?;
        manifest["gpu_timing"] = json!(if gpu.timing_enabled { "on" } else { "off" });
        manifest["timing_provenance"]["gpu_timing"] = json!(if gpu.timing_enabled {
            "normal: common Metal marker and timestamp harvesting enabled"
        } else {
            "calibration-only: begin marker omitted; common rendering/blit/presentation/retirement unchanged; GPU fields absent with calibration-disabled reason"
        });
        if !gpu.timing_enabled {
            manifest["timing_provenance"]["gpu_span_ms"] = json!("calibration-disabled");
            manifest["unsupported"]["gpu_span_ms"] = json!("calibration-disabled");
        }
        log_event(
            &config.output,
            &format!(
                "GPU_TIMING mode={}",
                if gpu.timing_enabled { "on" } else { "off" }
            ),
        )?;
        manifest["init_ms"] = json!(ms(init));
        manifest["fixture_hash"] = json!(fixture.spec_hash);
        manifest["identity_precompute_ms"] = json!(fixture.identity_precompute_ms);
        manifest["expected_live_state_sequence_hash"] =
            json!(fixture.expected_live_sequence_hash());
        manifest["input_hash"] = json!(fixture.input_hash);
        manifest["image_hash"] = json!(fixture.image_hash);
        manifest["gpu"] = json!({"name":gpu.name,"api":"Metal","software_fallback":false});
        manifest["completed_sample_count"] = json!(0);
        write_json(config.output.join("manifest.json"), &manifest)?;
        Ok(Self {
            backend,
            gpu,
            fixture,
            config,
            frames: vec![],
            pending: VecDeque::new(),
            manifest,
            finished: false,
            last_phase: None,
            next_index: 0,
        })
    }
    fn retire(&mut self) -> Result<f64, String> {
        let Some(i) = self.pending.pop_front() else {
            return Ok(0.0);
        };
        let t = Instant::now();
        let frame = &mut self.frames[i];
        let ticket = frame.ticket.take().unwrap();
        ticket.end.waitUntilCompleted();
        let wait = ms(t);
        if ticket.end.status() == MTLCommandBufferStatus::Error {
            return Err(format!("Metal command failed: {:?}", ticket.end.error()));
        }
        if let Some(begin) = &ticket.begin {
            let start = begin.GPUStartTime();
            let end = ticket.end.GPUEndTime();
            if start.is_finite() && end.is_finite() && start > 0.0 && end >= start {
                frame.gpu_start_s = Some(start);
                frame.gpu_end_s = Some(end);
                frame.gpu_span_ms = Some((end - start) * 1000.0);
            }
        }
        Ok(wait)
    }
    fn step(&mut self, phase: u32, index: u64) -> Result<(), String> {
        if self.finished {
            return Err("Trial already finished".into());
        }
        let phase_name = match phase {
            0 => "first-use",
            1 => "warmup",
            2 => "measured",
            _ => return Err("Invalid phase".into()),
        };
        if self.last_phase != Some(phase) {
            if index != 0 || self.last_phase.is_some_and(|p| phase <= p) {
                return Err("Phase must advance and reset frame index to zero".into());
            }
            self.last_phase = Some(phase);
            self.next_index = 0;
        }
        if index != self.next_index {
            return Err("Nonsequential frame index".into());
        }
        self.next_index += 1;
        let cpu_frame_start_s = now();
        let mut wait = 0.0;
        let limit = if self.config.mode == "paced" { 2 } else { 1 };
        while self.pending.len() >= limit {
            wait += self.retire()?;
        }
        let service = Instant::now();
        let changed = self.fixture.prepare(index as usize)?;
        let t = Instant::now();
        let drawable = self.gpu.acquire()?;
        let acquire_ms = ms(t);
        // Same timeline boundary for both adapters: all backend-specific CPU scene work,
        // encoding, uploads, submission and common blit. Common core geometry is outside.
        let begin = self.gpu.begin()?;
        let scene_ms = self
            .backend
            .prepare(&self.fixture, &self.gpu, index as usize, changed)?;
        let t = Instant::now();
        self.backend
            .render(&self.fixture, &self.gpu, index as usize)?;
        let render_submit_ms = ms(t);
        let cpu_submit_s = now();
        let t = Instant::now();
        let ticket = self.gpu.present(begin, drawable)?;
        let present_enqueue_ms = ms(t);
        let service_ms = ms(service);
        let state_hash =
            self.fixture
                .state_hash(index as usize, self.config.width, self.config.height);
        let i = self.frames.len();
        self.frames.push(Frame {
            phase: phase_name,
            index,
            state_hash,
            input_ms: self.fixture.input_ms,
            outline_ms: self.fixture.outline_ms,
            scene_ms,
            acquire_ms,
            render_submit_ms,
            present_enqueue_ms,
            completion_wait_ms: wait,
            service_ms,
            cpu_frame_start_s,
            cpu_submit_s,
            present_signal: Some(ticket.presented.clone()),
            ticket: Some(ticket),
            ..Default::default()
        });
        self.pending.push_back(i);
        if self.config.mode == "completion" {
            let w = self.retire()?;
            self.frames[i].completion_wait_ms += w;
        }
        Ok(())
    }
    fn finish(&mut self, status: &str, elapsed: f64) -> Result<(), String> {
        if self.finished {
            return Err("Trial already finished".into());
        }
        if !["complete", "cancelled", "invalidated", "failed"].contains(&status) {
            return Err("Invalid finish status".into());
        }
        log_event(
            &self.config.output,
            &format!(
                "FINISH requested_status={status} submitted_frames={} host_measured_elapsed_s={elapsed:.9}",
                self.frames.len()
            ),
        )?;
        let drain_start = Instant::now();
        while !self.pending.is_empty() {
            self.retire()?;
        }
        let drain_ms = ms(drain_start);
        // Presentation callback can lag GPU completion. Bound drain and preserve absent events.
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let missing = self.frames.iter().any(|f| {
                f.present_signal
                    .as_ref()
                    .is_some_and(|t| t.load(Ordering::Acquire) == 0)
            });
            if !missing || Instant::now() >= deadline {
                break;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        for f in &mut self.frames {
            if let Some(t) = &f.present_signal {
                let p = f64::from_bits(t.load(Ordering::Acquire));
                if p.is_finite() && p > 0.0 {
                    f.presented_s = Some(p);
                }
            }
        }
        let count = self.frames.iter().filter(|f| f.phase == "measured").count();
        if status == "complete"
            && (self
                .config
                .frames_requested
                .is_some_and(|n| count as u64 != n)
                || self
                    .config
                    .duration_seconds
                    .is_some_and(|n| !elapsed.is_finite() || elapsed < n)
                || self.frames.iter().filter(|f| f.phase == "warmup").count() as u64
                    != self.config.warmup)
        {
            return Err("Complete status rejected: incomplete sample coverage".into());
        }
        self.write_frames()?;
        if !self.frames.is_empty() {
            let last = self.frames.last().unwrap();
            let actual_state_hash = self.fixture.actual_state_hash(
                last.index as usize,
                self.config.width,
                self.config.height,
            );
            if actual_state_hash != last.state_hash {
                return Err(
                    "Final actual fixture geometry does not match recorded expected identity"
                        .into(),
                );
            }
            let rgba = self.gpu.readback()?;
            let output = BufWriter::new(
                File::create(self.config.output.join("final.png")).map_err(|e| e.to_string())?,
            );
            let mut enc = png::Encoder::new(output, self.config.width, self.config.height);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header()
                .map_err(|e| e.to_string())?
                .write_image_data(&rgba)
                .map_err(|e| e.to_string())?;
            let nonwhite = rgba
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|p| p[0] < 250 || p[1] < 250 || p[2] < 250)
                .count();
            let opaque = rgba.as_chunks::<4>().0.iter().all(|p| p[3] == 255);
            if nonwhite == 0 || !opaque {
                return Err("Readback failed basic nonblank/opaque-white-background check".into());
            }
            let last = self.frames.last().unwrap();
            write_json(
                self.config.output.join("readback.json"),
                &json!({"width":self.config.width,"height":self.config.height,"phase":last.phase,"frame_index":last.index,"state_hash":last.state_hash,"actual_state_hash":actual_state_hash,"identity_validation":"actual final geometry recomputed after timing and matched expected state","rgba_sha256":fixtures::hash(&rgba),"image_hash":self.fixture.image_hash,"nonwhite_pixels":nonwhite,"opaque_background":opaque,"validation":"basic structural check only; visual equivalence requires inspection"}),
            )?;
        }
        let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
        unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut usage) };
        self.manifest["status"] = json!(status);
        self.manifest["completed_sample_count"] = json!(count);
        self.manifest["measured_elapsed_s"] = json!(elapsed + drain_ms / 1000.0);
        self.manifest["measured_drain_ms"] = json!(drain_ms);
        self.manifest["os_peak_rss_bytes"] = json!(usage.ru_maxrss);
        self.manifest["instrumentation_storage"] = json!({"row_count":self.frames.len(),"row_capacity":self.frames.capacity(),"row_struct_bytes":std::mem::size_of::<Frame>(),"state_hash_capacity_bytes":self.frames.iter().map(|f|f.state_hash.capacity()).sum::<usize>(),"note":"Rows and callback AtomicU64 handles retained until output. Metal command buffers are dropped on retirement. RSS includes this bounded storage; sustained row count differs with throughput."});
        self.manifest["gpu_allocated_bytes_at_finish"] =
            json!(self.gpu.metal_device.currentAllocatedSize());
        if let Some(f) = self.frames.first() {
            self.manifest["first_use"] = json!({"input_ms":f.input_ms,"outline_ms":f.outline_ms,"scene_ms":f.scene_ms,"render_submit_ms":f.render_submit_ms,"service_ms":f.service_ms,"gpu_span_ms":f.gpu_span_ms});
        }
        log_event(
            &self.config.output,
            &format!(
                "FINISHED status={status} measured_samples={count} measured_elapsed_s={:.9} drain_ms={drain_ms:.6}",
                elapsed + drain_ms / 1000.0
            ),
        )?;
        write_json(self.config.output.join("manifest.json"), &self.manifest)?;
        self.finished = true;
        Ok(())
    }
    fn write_frames(&self) -> Result<(), String> {
        let mut w = BufWriter::new(
            File::create(self.config.output.join("frames.csv")).map_err(|e| e.to_string())?,
        );
        writeln!(w,"phase,frame_index,state_hash,input_ms,outline_ms,scene_ms,acquire_ms,render_submit_ms,present_enqueue_ms,completion_wait_ms,service_ms,cpu_frame_start_s,cpu_submit_s,gpu_span_ms,gpu_start_s,gpu_end_s,presented_s,gpu_missing_reason,present_missing_reason").map_err(|e|e.to_string())?;
        for f in &self.frames {
            let opt = |v: Option<f64>| v.map(|x| format!("{x:.9}")).unwrap_or_default();
            let gpu_reason = if !self.gpu.timing_enabled {
                "calibration-disabled"
            } else if f.gpu_span_ms.is_none() {
                "Metal timestamps unavailable or invalid"
            } else {
                ""
            };
            let present_reason = if f.presented_s.is_none() {
                "No positive presented callback within bounded drain"
            } else {
                ""
            };
            writeln!(w,"{},{},{},{:.9},{:.9},{:.9},{:.9},{:.9},{:.9},{:.9},{:.9},{:.9},{:.9},{},{},{},{},{},{}",f.phase,f.index,f.state_hash,f.input_ms,f.outline_ms,f.scene_ms,f.acquire_ms,f.render_submit_ms,f.present_enqueue_ms,f.completion_wait_ms,f.service_ms,f.cpu_frame_start_s,f.cpu_submit_s,opt(f.gpu_span_ms),opt(f.gpu_start_s),opt(f.gpu_end_s),opt(f.presented_s),gpu_reason,present_reason).map_err(|e|e.to_string())?;
        }
        w.flush().map_err(|e| e.to_string())
    }
}
thread_local! {static ERROR:RefCell<CString>=RefCell::new(CString::new("").unwrap());}
fn error(e: String) {
    ERROR.with(|v| *v.borrow_mut() = CString::new(e.replace('\0', " ")).unwrap());
}
fn ffi(f: impl FnOnce() -> Result<(), String>) -> i32 {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(Ok(())) => 0,
        Ok(Err(e)) => {
            error(e);
            1
        }
        Err(_) => {
            error("Rust panic in renderer experiment; trial must be abandoned".into());
            1
        }
    }
}
/// # Safety
/// Non-null config points to a valid NUL-terminated UTF-8 string, and layer to a live
/// CAMetalLayer. Layer lifecycle changes must be coordinated with the serial host queue.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rc_create(config: *const c_char, layer: *mut c_void) -> *mut c_void {
    let mut result = std::ptr::null_mut();
    ffi(|| {
        if config.is_null() || layer.is_null() {
            return Err("Null create argument".into());
        }
        let text = unsafe { CStr::from_ptr(config) }
            .to_str()
            .map_err(|e| e.to_string())?;
        let config = serde_json::from_str(text).map_err(|e| format!("Invalid config JSON: {e}"))?;
        let session = unsafe { Session::new(config, layer) }?;
        result = Box::into_raw(Box::new(session)).cast();
        Ok(())
    });
    result
}
/// # Safety
/// Handle is null or a live uniquely owned rc_create handle, used serially on its host queue.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rc_step(handle: *mut c_void, phase: u32, index: u64) -> i32 {
    let status = ffi(|| {
        unsafe { handle.cast::<Session>().as_mut() }
            .ok_or("Null handle")?
            .step(phase, index)
    });
    if status != 0 {
        ffi(|| {
            if let Some(s) = unsafe { handle.cast::<Session>().as_mut() } {
                let message = ERROR.with(|e| e.borrow().to_string_lossy().into_owned());
                s.record_failure("step", &message);
            }
            Ok(())
        });
    }
    status
}
/// # Safety
/// Handle is null or live and uniquely used. Non-null status points to a valid C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rc_finish(
    handle: *mut c_void,
    status: *const c_char,
    elapsed: f64,
) -> i32 {
    let result = ffi(|| {
        if status.is_null() {
            return Err("Null status".into());
        }
        let status = unsafe { CStr::from_ptr(status) }
            .to_str()
            .map_err(|e| e.to_string())?;
        unsafe { handle.cast::<Session>().as_mut() }
            .ok_or("Null handle")?
            .finish(status, elapsed)
    });
    if result != 0 {
        ffi(|| {
            if let Some(s) = unsafe { handle.cast::<Session>().as_mut() } {
                let message = ERROR.with(|e| e.borrow().to_string_lossy().into_owned());
                s.record_failure("finish", &message);
            }
            Ok(())
        });
    }
    result
}
/// # Safety
/// Handle is null or a live handle returned by rc_create, destroyed exactly once after
/// all concurrent host access has stopped.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rc_destroy(handle: *mut c_void) {
    if !handle.is_null() {
        ffi(|| {
            let mut s = unsafe { Box::from_raw(handle.cast::<Session>()) };
            while !s.pending.is_empty() {
                let _ = s.retire();
            }
            drop(s);
            Ok(())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn rc_error(_handle: *mut c_void) -> *const c_char {
    ERROR.with(|v| v.borrow().as_ptr())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_ambiguous_or_invalid_run_config() {
        let mut c:Config=serde_json::from_value(json!({"backend":"skia","fixture":"live","width":1600,"height":1000,"mode":"paced","warmup":120,"frames_requested":600,"repeat_id":"test","output":"/tmp/test"})).unwrap();
        assert!(c.validate().is_ok());
        c.duration_seconds = Some(180.0);
        assert!(c.validate().is_err());
        c.duration_seconds = None;
        c.output = PathBuf::from("relative");
        assert!(c.validate().is_err());
    }
}
