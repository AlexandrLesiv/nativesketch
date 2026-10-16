use super::work::NativeFailure;
use crate::storage::validate_directory;
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

/// Worker-only provisioning beneath trusted local ancestors. Existing directories are never
/// chmodded. Rechecks reject unexpected entries; this is not an adversarial TOCTOU guarantee.
pub(super) fn provision(path: &Path) -> Result<(), NativeFailure> {
    if !path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::CurDir))
    {
        return Err(NativeFailure::Access);
    }
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_dir() => {}
            Ok(_) => return Err(NativeFailure::Access),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let mut builder = fs::DirBuilder::new();
                #[cfg(unix)]
                {
                    use std::os::unix::fs::DirBuilderExt;
                    builder.mode(0o700);
                }
                builder
                    .create(&current)
                    .map_err(|_| NativeFailure::Access)?;
            }
            Err(_) => return Err(NativeFailure::Access),
        }
    }
    validate_directory(path).map_err(|_| NativeFailure::Access)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if fs::metadata(path)
            .map_err(|_| NativeFailure::Access)?
            .permissions()
            .mode()
            & 0o077
            != 0
        {
            return Err(NativeFailure::Access);
        }
    }
    Ok(())
}
