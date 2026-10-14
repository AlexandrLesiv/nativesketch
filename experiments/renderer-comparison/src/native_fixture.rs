//! Borrowed, experiment-only fixture bridge for native drawing APIs.
use crate::fixtures::{self, Fixture};
use std::{
    ffi::{CStr, CString, c_char, c_void},
    time::Instant,
};
use vello_common::kurbo::PathEl;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RcCommand {
    pub verb: u32,
    pub reserved: u32,
    pub points: [f64; 6],
}
#[repr(C)]
pub struct RcOutline {
    pub stroke_id: u64,
    pub command_start: u64,
    pub command_count: u64,
    pub rgba: [f64; 4],
}
#[repr(C)]
pub struct RcFixtureFrame {
    pub commands: *const RcCommand,
    pub command_count: u64,
    pub outlines: *const RcOutline,
    pub outline_count: u64,
    pub clip_commands: *const RcCommand,
    pub clip_command_count: u64,
    pub has_clip: u32,
    pub geometry_changed: u32,
    pub transform: [f64; 6],
    pub image_transform: [f64; 6],
    pub input_ms: f64,
    pub outline_ms: f64,
    pub export_ms: f64,
}
struct NativeFixture {
    fixture: Fixture,
    width: u32,
    height: u32,
    commands: Vec<RcCommand>,
    outlines: Vec<RcOutline>,
    clip: Vec<RcCommand>,
    string: CString,
    failed: bool,
    init_ms: f64,
}
fn clip_commands() -> Vec<RcCommand> {
    fixtures::clip_curve()
        .elements()
        .iter()
        .map(|e| match *e {
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
            PathEl::QuadTo(..) => unreachable!("ellipse uses cubics"),
        })
        .collect()
}
fn call(handle: *mut c_void, f: impl FnOnce(&mut NativeFixture) -> Result<(), String>) -> i32 {
    let status = crate::ffi(|| {
        let n = unsafe { handle.cast::<NativeFixture>().as_mut() }.ok_or("Null native fixture")?;
        if n.failed {
            return Err("Fixture failed; destroy it".into());
        }
        f(n)
    });
    if status != 0
        && let Some(n) = unsafe { handle.cast::<NativeFixture>().as_mut() }
    {
        n.failed = true;
    }
    status
}
/// # Safety
/// Pointer arguments must be null or valid for their documented pointee type.
/// Handles are uniquely owned, used serially, and destroyed exactly once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rc_fixture_create(
    name: *const c_char,
    width: u32,
    height: u32,
) -> *mut c_void {
    let mut result = std::ptr::null_mut();
    crate::ffi(|| {
        if name.is_null() || !(64..=8192).contains(&width) || !(64..=8192).contains(&height) {
            return Err("Invalid native fixture arguments".into());
        }
        let id = unsafe { CStr::from_ptr(name) }
            .to_str()
            .map_err(|e| e.to_string())?;
        let start = Instant::now();
        let mut fixture = Fixture::new(id)?;
        fixture.configure_identity(width, height);
        result = Box::into_raw(Box::new(NativeFixture {
            fixture,
            width,
            height,
            commands: vec![],
            outlines: vec![],
            clip: clip_commands(),
            string: CString::new("").unwrap(),
            failed: false,
            init_ms: start.elapsed().as_secs_f64() * 1000.,
        }))
        .cast();
        Ok(())
    });
    result
}
/// # Safety
/// Pointer arguments must be null or valid for their documented pointee type.
/// Handles are uniquely owned, used serially, and destroyed exactly once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rc_fixture_prepare(
    handle: *mut c_void,
    frame: u64,
    out: *mut RcFixtureFrame,
) -> i32 {
    call(handle, |n| {
        if out.is_null() {
            return Err("Null frame output".into());
        }
        let frame = usize::try_from(frame).map_err(|e| e.to_string())?;
        let changed = n.fixture.prepare(frame)?;
        let t = Instant::now();
        if changed {
            n.commands.clear();
            n.outlines.clear();
            for p in &n.fixture.paths {
                let start = n.commands.len();
                n.commands.extend(p.commands.iter().map(|c| RcCommand {
                    verb: c.verb,
                    reserved: 0,
                    points: c.points,
                }));
                n.outlines.push(RcOutline {
                    stroke_id: p.id,
                    command_start: start as u64,
                    command_count: p.commands.len() as u64,
                    rgba: p.rgba,
                });
            }
        }
        let export_ms = t.elapsed().as_secs_f64() * 1000.;
        unsafe {
            *out = RcFixtureFrame {
                commands: n.commands.as_ptr(),
                command_count: n.commands.len() as u64,
                outlines: n.outlines.as_ptr(),
                outline_count: n.outlines.len() as u64,
                clip_commands: n.clip.as_ptr(),
                clip_command_count: n.clip.len() as u64,
                has_clip: u32::from(n.fixture.id == "clip"),
                geometry_changed: u32::from(changed),
                transform: fixtures::transform(&n.fixture.id, frame, n.width, n.height),
                image_transform: fixtures::image_transform(frame),
                input_ms: n.fixture.input_ms,
                outline_ms: n.fixture.outline_ms,
                export_ms,
            }
        };
        Ok(())
    })
}
/// # Safety
/// Pointer arguments must be null or valid for their documented pointee type.
/// Handles are uniquely owned, used serially, and destroyed exactly once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rc_fixture_metadata_json(handle: *mut c_void) -> *const c_char {
    let mut result = std::ptr::null();
    call(handle, |n| {
        let mut clip = vec![];
        for c in &n.clip {
            clip.extend_from_slice(&c.verb.to_le_bytes());
            for p in c.points {
                clip.extend_from_slice(&p.to_le_bytes())
            }
        }
        n.string=CString::new(serde_json::json!({"fixture_hash":n.fixture.spec_hash,"input_hash":n.fixture.input_hash,"image_hash":n.fixture.image_hash,"clip_hash":fixtures::hash(&clip),"identity_precompute_ms":n.fixture.identity_precompute_ms,"fixture_init_ms":n.init_ms,"expected_live_state_sequence_hash":n.fixture.expected_live_sequence_hash(),"source_revision":option_env!("RENDERER_SOURCE_REVISION").unwrap_or("unrecorded"),"source_diff_hash":option_env!("RENDERER_SOURCE_DIFF_HASH").unwrap_or("unrecorded")}).to_string()).map_err(|e|e.to_string())?;
        result = n.string.as_ptr();
        Ok(())
    });
    result
}
/// # Safety
/// Pointer arguments must be null or valid for their documented pointee type.
/// Handles are uniquely owned, used serially, and destroyed exactly once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rc_fixture_state_hash(
    handle: *mut c_void,
    frame: u64,
    actual: u32,
) -> *const c_char {
    let mut result = std::ptr::null();
    call(handle, |n| {
        if actual > 1 {
            return Err("Invalid actual flag".into());
        }
        let frame = usize::try_from(frame).map_err(|e| e.to_string())?;
        let hash = if actual == 1 {
            n.fixture.actual_state_hash(frame, n.width, n.height)
        } else {
            n.fixture.state_hash(frame, n.width, n.height)
        };
        n.string = CString::new(hash).unwrap();
        result = n.string.as_ptr();
        Ok(())
    });
    result
}
/// # Safety
/// Pointer arguments must be null or valid for their documented pointee type.
/// Handles are uniquely owned, used serially, and destroyed exactly once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rc_fixture_image_rgba(handle: *mut c_void, count: *mut u64) -> *const u8 {
    let mut result = std::ptr::null();
    call(handle, |n| {
        if count.is_null() {
            return Err("Null image length".into());
        }
        unsafe { *count = n.fixture.image.len() as u64 };
        result = n.fixture.image.as_ptr();
        Ok(())
    });
    result
}
/// # Safety
/// Pointer arguments must be null or valid for their documented pointee type.
/// Handles are uniquely owned, used serially, and destroyed exactly once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rc_fixture_destroy(handle: *mut c_void) {
    if !handle.is_null() {
        crate::ffi(|| {
            drop(unsafe { Box::from_raw(handle.cast::<NativeFixture>()) });
            Ok(())
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ffi_identity_and_geometry_match_source() {
        unsafe {
            for id in fixtures::IDS {
                let s = CString::new(*id).unwrap();
                let h = rc_fixture_create(s.as_ptr(), 1600, 1000);
                assert!(!h.is_null());
                let mut out = std::mem::MaybeUninit::uninit();
                assert_eq!(rc_fixture_prepare(h, 0, out.as_mut_ptr()), 0);
                let f = out.assume_init();
                assert!(f.command_count > 0 && f.outline_count > 0);
                let expected = CStr::from_ptr(rc_fixture_state_hash(h, 0, 0)).to_owned();
                assert_eq!(
                    expected.as_c_str(),
                    CStr::from_ptr(rc_fixture_state_hash(h, 0, 1))
                );
                assert_eq!(rc_fixture_prepare(h, 1, &mut { f }), 0);
                rc_fixture_destroy(h);
            }
        }
    }
    #[test]
    fn null_and_poisoned_handles_fail() {
        unsafe {
            assert!(rc_fixture_create(std::ptr::null(), 1600, 1000).is_null());
            let name = CString::new("live").unwrap();
            let h = rc_fixture_create(name.as_ptr(), 1600, 1000);
            assert_eq!(rc_fixture_prepare(h, 0, std::ptr::null_mut()), 1);
            assert!(rc_fixture_metadata_json(h).is_null());
            rc_fixture_destroy(h);
        }
    }
    #[test]
    fn native_retention_and_live_reset() {
        unsafe {
            for id in ["live", "panzoom"] {
                let name = CString::new(id).unwrap();
                let h = rc_fixture_create(name.as_ptr(), 1600, 1000);
                let mut f = std::mem::MaybeUninit::uninit();
                assert_eq!(rc_fixture_prepare(h, 0, f.as_mut_ptr()), 0);
                let mut f = f.assume_init();
                let hash = CStr::from_ptr(rc_fixture_state_hash(h, 0, 1)).to_owned();
                let pointer = f.commands;
                assert_eq!(rc_fixture_prepare(h, 119, &mut f), 0);
                assert_eq!(f.geometry_changed, u32::from(id == "live"));
                if id == "panzoom" {
                    assert_eq!(pointer, f.commands)
                }
                assert_eq!(rc_fixture_prepare(h, 0, &mut f), 0);
                assert_eq!(
                    hash.as_c_str(),
                    CStr::from_ptr(rc_fixture_state_hash(h, 0, 1))
                );
                rc_fixture_destroy(h);
            }
        }
    }
}
