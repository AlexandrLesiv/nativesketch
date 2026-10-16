use crate::WireError;
use std::sync::{Arc, Mutex};
#[derive(Debug)]
pub(crate) struct Budget {
    maximum_count: usize,
    maximum_bytes: usize,
    used: Mutex<(usize, usize)>,
}
#[derive(Debug)]
pub(crate) struct Permit {
    budget: Arc<Budget>,
    bytes: usize,
}
impl Budget {
    pub fn new(count: usize, bytes: usize) -> Arc<Self> {
        Arc::new(Self {
            maximum_count: count,
            maximum_bytes: bytes,
            used: Mutex::new((0, 0)),
        })
    }
    pub fn reserve(self: &Arc<Self>, bytes: usize) -> Result<Permit, WireError> {
        let mut used = self.used.lock().map_err(|_| WireError::plain(10))?;
        if used.0 >= self.maximum_count || bytes > self.maximum_bytes.saturating_sub(used.1) {
            return Err(WireError::capacity());
        }
        used.0 += 1;
        used.1 += bytes;
        Ok(Permit {
            budget: self.clone(),
            bytes,
        })
    }
    pub fn stats(&self) -> (usize, usize) {
        *self
            .used
            .lock()
            .expect("budget has no panicking critical section")
    }
}
impl Drop for Permit {
    fn drop(&mut self) {
        let mut used = self
            .budget
            .used
            .lock()
            .expect("budget has no panicking critical section");
        used.0 -= 1;
        used.1 -= self.bytes;
    }
}
