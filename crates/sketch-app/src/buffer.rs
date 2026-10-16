use crate::*;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
#[derive(Debug)]
pub(crate) struct Budget {
    pub count: AtomicUsize,
    pub bytes: AtomicUsize,
    max_count: usize,
    max_bytes: usize,
}
impl Budget {
    pub fn new(limits: AppLimits) -> Arc<Self> {
        Arc::new(Self {
            count: AtomicUsize::new(0),
            bytes: AtomicUsize::new(0),
            max_count: limits.buffers,
            max_bytes: limits.buffer_bytes,
        })
    }
    // Only app owner reserves. Workers may release concurrently; CAS prevents lost updates.
    pub fn reserve(self: &Arc<Self>, bytes: usize) -> Result<Permit, AppError> {
        self.count
            .try_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                n.checked_add(1).filter(|n| *n <= self.max_count)
            })
            .map_err(|_| AppError::Capacity)?;
        if self
            .bytes
            .try_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                n.checked_add(bytes).filter(|n| *n <= self.max_bytes)
            })
            .is_err()
        {
            self.count.fetch_sub(1, Ordering::AcqRel);
            return Err(AppError::Capacity);
        }
        Ok(Permit {
            budget: Arc::clone(self),
            bytes,
        })
    }
}
#[derive(Debug)]
pub(crate) struct Permit {
    budget: Arc<Budget>,
    bytes: usize,
}
impl Permit {
    pub fn shrink(&mut self, bytes: usize) {
        assert!(bytes <= self.bytes);
        self.budget
            .bytes
            .fetch_sub(self.bytes - bytes, Ordering::AcqRel);
        self.bytes = bytes;
    }
}
impl Drop for Permit {
    fn drop(&mut self) {
        self.budget.bytes.fetch_sub(self.bytes, Ordering::AcqRel);
        self.budget.count.fetch_sub(1, Ordering::AcqRel);
    }
}
