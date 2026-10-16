use super::*;
#[test]
fn issued_write_classification_rejects_foreign_and_stale_ids() {
    let (mut a, handle) = app(AppLimits::default(), true);
    let write_id = save(&mut a, handle, resource(1), false);
    assert_eq!(a.operation_is_write(write_id), Ok(true));
    let (foreign, _) = app(AppLimits::default(), true);
    assert_eq!(foreign.operation_is_write(write_id), Err(AppError::Stale));
    let work = write(&mut a);
    assert_eq!(a.operation_is_write(write_id), Ok(true));
    complete(&mut a, written(work));
    assert_eq!(a.operation_is_write(write_id), Ok(true));
    a.apply_ready(handle, write_id).unwrap();
    assert_eq!(a.operation_is_write(write_id), Err(AppError::Stale));
    let read_id = open(&mut a, handle, resource(2));
    assert_eq!(a.operation_is_write(read_id), Ok(false));
    a.cancel_operation(read_id).unwrap();
    assert_eq!(a.operation_is_write(read_id), Err(AppError::Stale));
}
