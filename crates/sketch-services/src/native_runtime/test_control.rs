//! Explicit feature-gated disposable worker controls. Never built into normal production.
use super::work::NativeFailure;
use crate::storage::{Stage, StoreHooks, validate_directory};
use std::{
    fs::{self, OpenOptions},
    io,
    path::PathBuf,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
pub struct TestControl {
    root: PathBuf,
    stage: u32,
    hold: bool,
}
impl TestControl {
    pub fn new(root: PathBuf, stage: u32, mode: u32) -> Result<Self, NativeFailure> {
        if !root.is_absolute() || !(1..=9).contains(&stage) || !(1..=2).contains(&mode) {
            return Err(NativeFailure::Unsupported);
        }
        Ok(Self {
            root,
            stage,
            hold: mode == 1,
        })
    }
    pub(super) fn validate(&self) -> Result<(), NativeFailure> {
        validate_directory(&self.root).map_err(|_| NativeFailure::Access)?;
        if fs::read_dir(&self.root)
            .map_err(|_| NativeFailure::Access)?
            .next()
            .is_some()
        {
            return Err(NativeFailure::Access);
        }
        Ok(())
    }
}
#[derive(Default)]
pub(super) struct TestControls {
    control: Mutex<Option<TestControl>>,
}
impl TestControls {
    pub fn arm(&self, mut control: TestControl) -> Result<(), NativeFailure> {
        control.root = control
            .root
            .canonicalize()
            .map_err(|_| NativeFailure::Access)?;
        control.validate()?;
        let mut pending = self.control.lock().map_err(|_| NativeFailure::Panic)?;
        if pending.is_some() {
            return Err(NativeFailure::Capacity);
        }
        *pending = Some(control);
        Ok(())
    }
    fn reach(&self, stage: u32, stop: &AtomicBool) -> io::Result<()> {
        let control = {
            let mut pending = self.control.lock().map_err(|_| io::ErrorKind::Other)?;
            if !pending
                .as_ref()
                .is_some_and(|control| control.stage == stage)
            {
                return Ok(());
            }
            pending.take().expect("matching control")
        };
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        options.open(control.root.join("reached"))?;
        if !control.hold {
            return Err(io::ErrorKind::Other.into());
        }
        let started = Instant::now();
        loop {
            if stop.load(Ordering::Acquire) || started.elapsed() >= Duration::from_secs(60) {
                return Err(io::ErrorKind::Interrupted.into());
            }
            match fs::symlink_metadata(control.root.join("release")) {
                Ok(metadata) if metadata.is_file() => return Ok(()),
                Ok(_) => return Err(io::ErrorKind::InvalidInput.into()),
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    pub fn after_read_validation(&self, stop: &AtomicBool) -> io::Result<()> {
        self.reach(8, stop)
    }
    pub fn resolve(&self, stop: &AtomicBool) -> io::Result<()> {
        self.reach(1, stop)
    }
    pub fn hooks<'a>(&'a self, stop: &'a AtomicBool) -> ControlledHooks<'a> {
        ControlledHooks {
            controls: self,
            stop,
        }
    }
}
pub(super) struct ControlledHooks<'a> {
    controls: &'a TestControls,
    stop: &'a AtomicBool,
}
impl StoreHooks for ControlledHooks<'_> {
    fn completed(
        &self,
        outcome: sketch_core::application::storage::WriteOutcome,
    ) -> sketch_core::application::storage::WriteOutcome {
        let corrupt = self
            .controls
            .control
            .lock()
            .is_ok_and(|control| control.as_ref().is_some_and(|c| c.stage == 9));
        if corrupt && self.controls.reach(9, self.stop).is_ok() {
            sketch_core::application::storage::WriteOutcome::DispatchedUnconfirmed
        } else {
            outcome
        }
    }
    fn at(&self, stage: Stage) -> io::Result<()> {
        let stage = match stage {
            Stage::BeforeCreate => 2,
            Stage::AfterWrite => 3,
            Stage::AfterFileSync => 4,
            Stage::BeforeReplace => 5,
            Stage::AfterReplace => 6,
            Stage::BeforeDirectorySync => 7,
            _ => return Ok(()),
        };
        self.controls.reach(stage, self.stop)
    }
}
