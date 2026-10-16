use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

/// Trusted host supplies a local absolute path/access grant. Never put this in diagnostics.
/// Recheck before each write; symlinks and non-regular destinations are rejected.
pub struct DestinationAccess {
    path: PathBuf,
}
impl DestinationAccess {
    pub fn new(path: PathBuf) -> io::Result<Self> {
        let access = Self { path };
        access.validate()?;
        Ok(access)
    }
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
    pub(crate) fn validate(&self) -> io::Result<()> {
        let parent = self.path.parent().ok_or(io::ErrorKind::InvalidInput)?;
        validate_directory(parent)?;
        if self.path.file_name().is_none() {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        match fs::symlink_metadata(&self.path) {
            Ok(meta) if !meta.file_type().is_file() => Err(io::ErrorKind::InvalidInput.into()),
            Ok(_) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }
}
pub(crate) fn validate_directory(path: &Path) -> io::Result<()> {
    if !path.is_absolute() {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    let mut current = PathBuf::new();
    for component in path.components() {
        if matches!(component, Component::ParentDir | Component::CurDir) {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        current.push(component.as_os_str());
        let metadata = fs::symlink_metadata(&current)?;
        if !metadata.file_type().is_dir() {
            return Err(io::ErrorKind::InvalidInput.into());
        }
    }
    Ok(())
}
