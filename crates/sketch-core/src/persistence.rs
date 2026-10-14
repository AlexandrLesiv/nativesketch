use super::*;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU64, Ordering};
const MAGIC: &[u8; 8] = b"NSKETCH1";
const MAX_BYTES: u64 = 64 * 1024 * 1024;
static TEMP_ID: AtomicU64 = AtomicU64::new(0);
fn checksum(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    })
}
fn put(bytes: &mut Vec<u8>, n: u64) {
    bytes.extend_from_slice(&n.to_le_bytes());
}
fn encode(document: &Document) -> Vec<u8> {
    let mut b = MAGIC.to_vec();
    put(&mut b, document.strokes.len() as u64);
    for s in &document.strokes {
        put(&mut b, s.id);
        put(&mut b, u64::from(s.device));
        put(&mut b, s.pointer_id);
        for x in s.brush.rgba.into_iter().chain([
            s.brush.width,
            s.transform.scale,
            s.transform.tx,
            s.transform.ty,
        ]) {
            put(&mut b, x.to_bits());
        }
        put(&mut b, s.samples.len() as u64);
        for p in &s.samples {
            put(&mut b, p.id);
            for x in [p.x, p.y, p.timestamp] {
                put(&mut b, x.to_bits());
            }
            put(&mut b, u64::from(p.pressure.is_some()));
            put(&mut b, p.pressure.unwrap_or(0.0).to_bits());
        }
    }
    let hash = checksum(&b);
    put(&mut b, hash);
    b
}
struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}
impl Reader<'_> {
    fn u64(&mut self) -> Result<u64> {
        let end = self.pos.checked_add(8).ok_or(Error::Format)?;
        let b = self.data.get(self.pos..end).ok_or(Error::Format)?;
        self.pos = end;
        Ok(u64::from_le_bytes(b.try_into().map_err(|_| Error::Format)?))
    }
    fn f64(&mut self) -> Result<f64> {
        Ok(f64::from_bits(self.u64()?))
    }
}
fn decode(bytes: &[u8]) -> Result<Document> {
    if bytes.len() < 24 || &bytes[..8] != MAGIC {
        return Err(Error::Format);
    }
    let end = bytes.len() - 8;
    let expected = u64::from_le_bytes(bytes[end..].try_into().map_err(|_| Error::Format)?);
    if checksum(&bytes[..end]) != expected {
        return Err(Error::Format);
    }
    let mut r = Reader {
        data: &bytes[..end],
        pos: 8,
    };
    let count = r.u64()?;
    if count > MAX_STROKES as u64 {
        return Err(Error::Format);
    }
    let mut strokes = Vec::new();
    let mut ids = std::collections::BTreeSet::new();
    for _ in 0..count {
        let id = r.u64()?;
        if id == 0 || id == u64::MAX || !ids.insert(id) {
            return Err(Error::Format);
        }
        let device = r.u64()?;
        if device > 2 {
            return Err(Error::Format);
        }
        let pointer_id = r.u64()?;
        let brush = Brush {
            rgba: [r.f64()?, r.f64()?, r.f64()?, r.f64()?],
            width: r.f64()?,
        };
        brush.validate().map_err(|_| Error::Format)?;
        let transform = Transform {
            scale: r.f64()?,
            tx: r.f64()?,
            ty: r.f64()?,
        };
        transform.validate().map_err(|_| Error::Format)?;
        let n = r.u64()?;
        if n == 0 || n > MAX_SAMPLES as u64 {
            return Err(Error::Format);
        }
        let mut samples: Vec<Sample> = Vec::new();
        for _ in 0..n {
            let id = r.u64()?;
            let x = r.f64()?;
            let y = r.f64()?;
            let timestamp = r.f64()?;
            let flag = r.u64()?;
            let p = r.f64()?;
            if flag > 1 || !p.is_finite() || flag == 0 && p != 0.0 {
                return Err(Error::Format);
            }
            let sample = Sample {
                id,
                x,
                y,
                timestamp,
                pressure: if flag == 1 { Some(p) } else { None },
            };
            sample.validate().map_err(|_| Error::Format)?;
            if samples
                .last()
                .is_some_and(|s| sample.id <= s.id || sample.timestamp < s.timestamp)
            {
                return Err(Error::Format);
            }
            samples.push(sample);
        }
        let stroke = Stroke {
            id,
            device: device as u32,
            pointer_id,
            brush,
            transform,
            samples,
        };
        let b = bounds(&stroke, transform);
        if !coordinate(b.x) || !coordinate(b.y) || !coordinate(b.width) || !coordinate(b.height) {
            return Err(Error::Format);
        }
        strokes.push(stroke);
    }
    if r.pos != end {
        return Err(Error::Format);
    }
    Ok(Document { strokes })
}
pub(super) fn save(document: &Document, path: &Path) -> Result<()> {
    save_with(document, path, |_| Ok(()))
}
fn save_with(
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
pub(super) fn load(path: &Path) -> Result<Document> {
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interrupted_replace_preserves_good_file() {
        let dir = std::env::temp_dir().join(format!(
            "sketch-persistence-{}",
            TEMP_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("doc.nsketch");
        fs::write(&path, b"last good").unwrap();
        let error = save_with(&Document::default(), &path, |temp| {
            assert!(temp.exists());
            Err(std::io::Error::other("injected interruption"))
        });
        assert_eq!(error, Err(Error::Io));
        assert_eq!(fs::read(&path).unwrap(), b"last good");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn checksum_and_version_reject_corruption() {
        let b = encode(&Document::default());
        assert_eq!(decode(&b).unwrap(), Document::default());
        for i in 0..b.len() {
            let mut c = b.clone();
            c[i] ^= 1;
            assert_eq!(decode(&c), Err(Error::Format));
        }
        for i in 0..b.len() {
            assert_eq!(decode(&b[..i]), Err(Error::Format));
        }
    }
    #[test]
    fn accepted_boundary_samples_roundtrip_from_default_editor() {
        let mut editor = Editor::default();
        let brush = Brush {
            rgba: [0.0, 0.0, 0.0, 1.0],
            width: 10000.0,
        };
        editor.begin(1, 0, brush).unwrap();
        let sample = Sample {
            id: 1,
            x: 1e9,
            y: -1e9,
            timestamp: 0.0,
            pressure: None,
        };
        editor.append(1, &[Input::Actual(sample)]).unwrap();
        editor.commit(1).unwrap();
        assert_eq!(editor.document.strokes[0].id, 1);
        assert_eq!(
            decode(&encode(editor.document())).unwrap(),
            *editor.document()
        );
        editor.select_cycle(1).unwrap();
        editor.resize_begin().unwrap();
        editor.resize_update(10.0, 0.0, 0.0).unwrap();
        editor.resize_commit().unwrap();
        assert_eq!(
            decode(&encode(editor.document())).unwrap(),
            *editor.document()
        );
        editor.begin(2, 0, brush).unwrap();
        let invalid = Sample { x: 1e12, ..sample };
        assert_eq!(
            editor.append(2, &[Input::Actual(invalid)]),
            Err(Error::InvalidArgument)
        );
    }
}
