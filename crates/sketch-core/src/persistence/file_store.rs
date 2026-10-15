use super::{MAX_BYTES, decode, encode};
use crate::{Document, Error, Result};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
pub(super) static TEMP_ID: AtomicU64 = AtomicU64::new(0);
pub(crate) fn save(document: &Document, path: &Path) -> Result<()> {
    save_with(document, path, |_| Ok(()))
}
pub(super) fn save_with(
    document: &Document,
    path: &Path,
    before_replace: impl FnOnce(&Path) -> std::io::Result<()>,
) -> Result<()> {
    let bytes = encode(document);
    if bytes.len() as u64 > MAX_BYTES {
        return Err(Error::InvalidArgument);
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = path
        .file_name()
        .ok_or(Error::InvalidArgument)?
        .to_string_lossy();
    let temp = parent.join(format!(
        ".{name}.{}.{}.tmp",
        std::process::id(),
        TEMP_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|_| Error::Io)?;
    let result = (|| {
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        before_replace(&temp)?;
        fs::rename(&temp, path)
    })()
    .map_err(|_| Error::Io);
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    // No fallible operation follows successful replacement: a returned failure
    // therefore always preserves the previous destination. Directory fsync is a
    // best-effort durability improvement; full power-loss guarantees are deferred.
    if result.is_ok() {
        if let Ok(dir) = fs::File::open(parent) {
            let _ = dir.sync_all();
        }
    }
    result
}
pub(crate) fn load(path: &Path) -> Result<Document> {
    let mut file = fs::File::open(path).map_err(|_| Error::Io)?;
    let mut bytes = Vec::new();
    (&mut file)
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Error::Io)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(Error::Format);
    }
    decode(&bytes)
}
