use crate::AppError;
macro_rules! handle {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name(pub(crate) u64, pub(crate) u64);
        impl $name {
            pub fn get(self) -> u64 {
                self.0
            }
        }
    };
}
handle!(SessionHandle);
handle!(OperationId);
handle!(DestinationHandle);
pub(crate) fn reserve(next: &mut u64) -> Result<u64, AppError> {
    let id = *next;
    *next = id.checked_add(1).ok_or(AppError::Capacity)?;
    Ok(id)
}
