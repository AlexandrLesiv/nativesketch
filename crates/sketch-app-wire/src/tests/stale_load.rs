use super::*;
#[test]
fn stale_loaded_candidate_retires_and_file_intents_recover() {
    let mut a = app();
    let id = session(&mut a);
    draw(&mut a, id);
    let export = queued(&mut a, id, 3, 1, 7);
    let mut encoded = take(&mut a);
    drop(encoded.call(&packet(101, &[]), &mut None).unwrap());
    let copied = encoded
        .call(
            &packet(107, &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0]),
            &mut None,
        )
        .unwrap();
    let bytes = copied.reply.bytes()[80..].to_vec();
    drop(copied);
    let mut result = encoded
        .call(&packet(104, &[1, 0, 0, 0, 3, 0, 0, 0]), &mut None)
        .unwrap()
        .result;
    drop(a.complete(&packet(31, &[]), &mut result).unwrap());
    drop(a.call(&packet(32, &words(&[id, export]))).unwrap());
    drop(a.call(&packet(34, &[])).unwrap());
    let r = file(&mut a, id, 5);
    let effect = u64_at(r.bytes(), 216);
    drop(r);
    let mut b = context(&mut a, id);
    b.extend(words(&[effect]));
    b.extend([3, 0, 0, 0, 0, 0, 0, 0]);
    let r = a.call(&packet(21, &b)).unwrap();
    let effect = u64_at(r.bytes(), 216);
    drop(r);
    let r = supply(&mut a, id, effect, 2, 2, 1);
    let op = u64_at(r.bytes(), 248);
    drop(r);
    let mut read = take(&mut a);
    let mut input = Some(read.copy_input(&bytes).unwrap());
    let mut result = read.call(&packet(102, &[]), &mut input).unwrap().result;
    draw(&mut a, id);
    drop(a.complete(&packet(31, &[]), &mut result).unwrap());
    assert_eq!(
        a.call(&packet(32, &words(&[id, op]))).unwrap_err().status,
        7
    );
    drain(&mut a);
    let r = file(&mut a, id, 2);
    assert_eq!(u32_at(r.bytes(), 232), 1);
}
