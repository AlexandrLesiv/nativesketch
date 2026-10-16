use super::*;
#[test]
fn wrong_kind_and_foreign_input_preserve_reads_then_corrupt_result_retires() {
    let _serial = serial();
    let a = app();
    let s = session(a);
    let op = queued(a, s, 5);
    let w = take(a);
    let other = app();
    let t = session(other);
    let other_op = queued(other, t, 5);
    let other_w = take(other);
    let input = work_copy_input(w, b"corrupt document").unwrap();
    assert_eq!(
        work_call(other_w, &packet(102, &[]), input)
            .unwrap_err()
            .status,
        11
    );
    assert_eq!(
        work_call(w, &packet(100, &[]), input).unwrap_err().status,
        1
    );
    let o = work_call(w, &packet(102, &[]), input).unwrap();
    assert!(o.work_consumed && o.input_consumed);
    buffer_free(o.reply).unwrap();
    assert_eq!(input_free(input).unwrap_err().status, 7);
    complete(a, o.result);
    call(a, &packet(32, &words(&[s, op])));
    let o = work_call(other_w, &packet(105, &[1, 0, 0, 0, 0, 0, 0, 0]), 0).unwrap();
    buffer_free(o.reply).unwrap();
    complete(other, o.result);
    call(other, &packet(32, &words(&[t, other_op])));
    close(a, s);
    close(other, t);
}
