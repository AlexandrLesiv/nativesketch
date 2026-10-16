use crate::{bridge, receipt::error};
use jni::{Env, objects::JByteBuffer};
use std::{marker::PhantomData, ptr::NonNull};
/// Validated memory extent pinned by the caller's JNI local reference.
/// Exclusive access is a caller obligation; overlap checks cannot prevent Java races.
#[derive(Clone, Copy)]
pub(crate) struct Region<'a> {
    pointer: *mut u8,
    length: usize,
    _borrow: PhantomData<&'a JByteBuffer<'a>>,
}
impl<'a> Region<'a> {
    pub fn resolve(
        env: &mut Env<'_>,
        buffer: &'a JByteBuffer<'_>,
        length: i64,
        maximum: usize,
        writable: bool,
    ) -> bridge::Result<Self> {
        let length = length_checked(length, maximum)?;
        let capacity = env
            .get_direct_buffer_capacity(buffer)
            .map_err(|_| error(1))?;
        if length > capacity {
            return Err(error(1));
        }
        if writable
            && env
                .call_method(
                    buffer,
                    jni::jni_str!("isReadOnly"),
                    jni::jni_sig!("()Z"),
                    &[],
                )
                .and_then(|v| v.z())
                .map_err(|_| error(1))?
        {
            return Err(error(1));
        }
        let pointer = if capacity == 0 {
            NonNull::<u8>::dangling().as_ptr()
        } else {
            env.get_direct_buffer_address(buffer)
                .map_err(|_| error(1))?
        };
        checked_extent(pointer as usize, length)?;
        Ok(Self {
            pointer,
            length,
            _borrow: PhantomData,
        })
    }
    pub fn disjoint(self, other: Self) -> bridge::Result<()> {
        if overlap(
            self.pointer as usize,
            self.length,
            other.pointer as usize,
            other.length,
        ) {
            Err(error(1))
        } else {
            Ok(())
        }
    }
    /// # Safety
    /// Caller prevents Java mutation while borrowed. All writable extents are disjoint.
    pub unsafe fn read(self) -> &'a [u8] {
        // Safety: resolve validated the extent; the caller holds its JNI ref and excludes writes.
        unsafe { std::slice::from_raw_parts(self.pointer, self.length) }
    }
    /// # Safety
    /// Caller grants exclusive access to this prevalidated writable direct region.
    pub unsafe fn write(self) -> &'a mut [u8] {
        // Safety: resolve checked writable memory; the caller excludes all concurrent access.
        unsafe { std::slice::from_raw_parts_mut(self.pointer, self.length) }
    }
    /// # Safety
    /// This is the preflighted 128-byte receipt, disjoint from all borrowed arguments.
    pub unsafe fn publish(self, bytes: &[u8; 128]) {
        // Safety: stack bytes cannot alias the pinned Java receipt, which has 128 writable bytes.
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), self.pointer, 128) };
    }
}
pub(crate) fn length_checked(value: i64, maximum: usize) -> bridge::Result<usize> {
    let n = usize::try_from(value).map_err(|_| error(1))?;
    if n > isize::MAX as usize {
        return Err(error(1));
    }
    if n > maximum {
        return Err(error(3));
    }
    Ok(n)
}
pub(crate) fn checked_extent(address: usize, length: usize) -> bridge::Result<()> {
    if address == 0 || length > isize::MAX as usize || address.checked_add(length).is_none() {
        Err(error(1))
    } else {
        Ok(())
    }
}
pub(crate) fn overlap(a: usize, n: usize, b: usize, m: usize) -> bool {
    n != 0 && m != 0 && a < b + m && b < a + n
}
