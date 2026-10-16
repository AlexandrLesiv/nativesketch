use super::{PreparationLimits, SceneError};
/// Counts allocated vector capacity, not merely visible output length. Allocator overhead
/// is excluded; stack scratch and the constant result header are bounded separately.
pub(super) struct Budget {
    pub limits: PreparationLimits,
    bytes: usize,
    work: usize,
    points: usize,
}
impl Budget {
    pub fn new(limits: PreparationLimits) -> Result<Self, SceneError> {
        if limits.points > 262_144
            || limits.depth > 32
            || limits.work > 8_388_608
            || limits.bytes < 256
        {
            return Err(SceneError::ResourceLimit);
        }
        Ok(Self {
            limits,
            bytes: 256,
            work: 0,
            points: 0,
        })
    }
    pub fn tick(&mut self) -> Result<(), SceneError> {
        if self.work == self.limits.work {
            return Err(SceneError::ResourceLimit);
        }
        self.work += 1;
        Ok(())
    }
    pub fn point(&mut self) -> Result<(), SceneError> {
        if self.points == self.limits.points {
            return Err(SceneError::ResourceLimit);
        }
        self.points += 1;
        Ok(())
    }
    pub fn push<T>(&mut self, values: &mut Vec<T>, value: T) -> Result<(), SceneError> {
        if values.len() == values.capacity() {
            let old = values.capacity() * size_of::<T>();
            let count = (values.capacity() / 2).clamp(1, 1024);
            let additional = count
                .checked_mul(size_of::<T>())
                .ok_or(SceneError::ResourceLimit)?;
            if additional > self.limits.bytes.saturating_sub(self.bytes) {
                return Err(SceneError::ResourceLimit);
            }
            values
                .try_reserve_exact(count)
                .map_err(|_| SceneError::ResourceLimit)?;
            self.bytes = self
                .bytes
                .checked_add(values.capacity() * size_of::<T>() - old)
                .ok_or(SceneError::ResourceLimit)?;
            if self.bytes > self.limits.bytes {
                return Err(SceneError::ResourceLimit);
            }
        }
        values.push(value);
        Ok(())
    }
    pub fn release<T>(&mut self, values: Vec<T>) {
        self.bytes -= values.capacity() * size_of::<T>();
        drop(values);
    }
}
