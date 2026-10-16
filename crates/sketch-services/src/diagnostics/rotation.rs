use super::Event;
use crate::storage::DestinationAccess;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
/// Fixed private filenames; bounded size/count. This sink runs on a worker, never ink callbacks.
pub struct RotatingSink {
    root: PathBuf,
    bytes: u64,
    files: usize,
}
impl RotatingSink {
    pub fn new(root: PathBuf, bytes: u64, files: usize) -> io::Result<Self> {
        if !(512..=1024 * 1024).contains(&bytes) || !(1..=8).contains(&files) {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        let sink = Self { root, bytes, files };
        sink.validate()?;
        Ok(sink)
    }
    fn validate(&self) -> io::Result<()> {
        for index in 0..self.files {
            DestinationAccess::new(self.path(index))?;
            match fs::metadata(self.path(index)) {
                Ok(metadata) if metadata.len() > self.bytes => {
                    return Err(io::ErrorKind::InvalidData.into());
                }
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }
    fn path(&self, index: usize) -> PathBuf {
        self.root.join(format!("diagnostics.{index}.jsonl"))
    }
    pub fn write(&mut self, event: Event) -> io::Result<()> {
        let line = event.encode();
        self.validate()?;
        if line.len() as u64 > self.bytes {
            return Err(io::ErrorKind::InvalidData.into());
        }
        let length = match fs::metadata(self.path(0)) {
            Ok(meta) => meta.len(),
            Err(error) if error.kind() == io::ErrorKind::NotFound => 0,
            Err(error) => return Err(error),
        };
        if length.saturating_add(line.len() as u64) > self.bytes {
            self.rotate()?;
        }
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.path(0))?
            .write_all(line.as_bytes())
    }
    fn rotate(&self) -> io::Result<()> {
        match fs::remove_file(self.path(self.files - 1)) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        for index in (0..self.files - 1).rev() {
            match fs::rename(self.path(index), self.path(index + 1)) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }
}
