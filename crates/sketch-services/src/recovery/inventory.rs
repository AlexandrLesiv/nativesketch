use super::{MAX_ENVELOPE, RecoveryError, RecoveryId, RecoveryLimits};
use crate::storage::validate_directory;
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};
pub(super) struct SessionFiles {
    pub id: RecoveryId,
    pub files: Vec<(u64, PathBuf)>,
    pub bytes: u64,
}
pub(super) fn inventory(
    root: &Path,
    limits: RecoveryLimits,
) -> Result<Vec<SessionFiles>, RecoveryError> {
    validate_directory(root).map_err(|_| RecoveryError::UnsafeEntry)?;
    let mut sessions = Vec::new();
    for entry in fs::read_dir(root)? {
        if sessions.len() >= limits.sessions {
            return Err(RecoveryError::Capacity);
        }
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            return Err(RecoveryError::UnsafeEntry);
        }
        let id = RecoveryId::parse(
            entry
                .file_name()
                .to_str()
                .ok_or(RecoveryError::UnsafeEntry)?,
        )?;
        sessions.push(scan_session(entry.path(), id)?);
    }
    Ok(sessions)
}
pub(super) fn scan_session(path: PathBuf, id: RecoveryId) -> Result<SessionFiles, RecoveryError> {
    validate_directory(&path).map_err(|_| RecoveryError::UnsafeEntry)?;
    let mut session = SessionFiles {
        id,
        files: Vec::new(),
        bytes: 0,
    };
    for (count, entry) in fs::read_dir(path)?.enumerate() {
        if count >= 64 {
            return Err(RecoveryError::Capacity);
        }
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            return Err(RecoveryError::UnsafeEntry);
        }
        let name = entry.file_name();
        let name = name.to_str().ok_or(RecoveryError::UnsafeEntry)?;
        session.bytes = session
            .bytes
            .checked_add(entry.metadata()?.len())
            .ok_or(RecoveryError::Capacity)?;
        if name == "resolved" || name.starts_with(".native-sketch-") && name.ends_with(".tmp") {
            continue;
        }
        let generation = name
            .strip_suffix(".nskr")
            .filter(|n| {
                n.len() == 16
                    && n.bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            })
            .and_then(|n| u64::from_str_radix(n, 16).ok())
            .filter(|n| *n > 0)
            .ok_or(RecoveryError::UnsafeEntry)?;
        session.files.push((generation, entry.path()));
    }
    session
        .files
        .sort_by_key(|(generation, _)| std::cmp::Reverse(*generation));
    Ok(session)
}
pub(super) fn read_bounded(path: &Path) -> Result<Vec<u8>, RecoveryError> {
    if !fs::symlink_metadata(path)?.file_type().is_file() {
        return Err(RecoveryError::UnsafeEntry);
    }
    if fs::metadata(path)?.len() > MAX_ENVELOPE {
        return Err(RecoveryError::Capacity);
    }
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(MAX_ENVELOPE + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_ENVELOPE {
        return Err(RecoveryError::Capacity);
    }
    Ok(bytes)
}
