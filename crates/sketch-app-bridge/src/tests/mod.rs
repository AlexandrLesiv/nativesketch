use crate::*;
mod lifecycle;
mod limits;
mod ownership;
mod read;
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());
fn serial() -> std::sync::MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|p| p.into_inner())
}
fn fixture(name: &str) -> Vec<u8> {
    let line = include_str!("../../../sketch-app-wire/tests/fixtures/v2.json")
        .lines()
        .find(|l| l.trim_start().starts_with(&format!("\"{name}\":")))
        .unwrap();
    let hex = line.split('"').nth(3).unwrap();
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
fn packet(op: u32, body: &[u8]) -> Vec<u8> {
    let mut p = b"NSAP\x02\0\0\0".to_vec();
    p.extend(op.to_le_bytes());
    p.extend(u32::try_from(body.len()).unwrap().to_le_bytes());
    p.extend(17u64.to_le_bytes());
    p.extend(body);
    p
}
fn words(values: &[u64]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}
fn u64_at(b: &[u8], n: usize) -> u64 {
    u64::from_le_bytes(b[n..n + 8].try_into().unwrap())
}
fn u32_at(b: &[u8], n: usize) -> u32 {
    u32::from_le_bytes(b[n..n + 4].try_into().unwrap())
}
fn copy_free(r: Handle) -> Vec<u8> {
    let mut bytes = vec![0; buffer_len(r).unwrap()];
    assert_eq!(buffer_copy(r, 0, &mut bytes).unwrap(), bytes.len());
    buffer_free(r).unwrap();
    bytes
}
fn app() -> Handle {
    app_new(&fixture("bootstrap")).unwrap()
}
fn call(a: Handle, p: &[u8]) -> Vec<u8> {
    copy_free(app_call(a, p).unwrap())
}
fn session(a: Handle) -> u64 {
    u64_at(&call(a, &fixture("create.cap3")), 72)
}
fn context(a: Handle, s: u64) -> Vec<u8> {
    let r = call(a, &packet(3, &words(&[s])));
    words(&[s, u64_at(&r, 80), u64_at(&r, 104)])
}
fn file(a: Handle, s: u64, tag: u32) -> Vec<u8> {
    let mut b = context(a, s);
    b.extend(tag.to_le_bytes());
    b.extend(0u32.to_le_bytes());
    call(a, &packet(20, &b))
}
fn queued(a: Handle, s: u64, tag: u32) -> u64 {
    let e = u64_at(&file(a, s, tag), 240);
    let mut b = context(a, s);
    b.extend(words(&[e]));
    b.extend(1u32.to_le_bytes());
    b.extend(7u32.to_le_bytes());
    b.extend(words(&[1, 1, 1]));
    u64_at(&call(a, &packet(22, &b)), 248)
}
fn take(a: Handle) -> Handle {
    let (r, w) = app_take_work(a, &packet(30, &[])).unwrap();
    buffer_free(r).unwrap();
    assert_ne!(w, 0);
    w
}
fn finish(w: Handle) -> Handle {
    let o = work_call(w, &packet(101, &[]), 0).unwrap();
    assert!(!o.work_consumed);
    buffer_free(o.reply).unwrap();
    let o = work_call(w, &packet(104, &[1, 0, 0, 0, 3, 0, 0, 0]), 0).unwrap();
    assert!(o.work_consumed);
    buffer_free(o.reply).unwrap();
    o.result
}
fn complete(a: Handle, r: Handle) {
    buffer_free(app_complete(a, &packet(31, &[]), r).unwrap()).unwrap();
}
fn close(a: Handle, s: u64) {
    let mut r = file(a, s, 6);
    if u32_at(&r, 232) == 3 {
        r = decision(a, s, u64_at(&r, 240), 3);
    }
    assert_eq!(u32_at(&r, 232), 6);
    let r = decision(a, s, u64_at(&r, 240), 5);
    assert_eq!(u32_at(&r, 232), 7);
    call(a, &packet(2, &words(&[s])));
    loop {
        let (r, w) = app_take_work(a, &packet(30, &[])).unwrap();
        buffer_free(r).unwrap();
        if w == 0 {
            break;
        }
        let o = work_call(w, &packet(106, &[]), 0).unwrap();
        buffer_free(o.reply).unwrap();
        complete(a, o.result);
    }
    while u32_at(&call(a, &packet(34, &[])), 72) != 0 {}
    app_free(a).unwrap();
}
fn draw(a: Handle, s: u64) {
    let mut b = context(a, s);
    b.extend(6u32.to_le_bytes());
    b.extend(0u32.to_le_bytes());
    b.extend(words(&[7]));
    b.extend(2u32.to_le_bytes());
    b.extend(0u32.to_le_bytes());
    call(a, &packet(10, &b));
    let mut b = context(a, s);
    b.extend_from_slice(&fixture("sample.0.1")[48..]);
    call(a, &packet(12, &b));
    let mut b = context(a, s);
    b.extend(7u32.to_le_bytes());
    b.extend(0u32.to_le_bytes());
    b.extend(words(&[7]));
    call(a, &packet(10, &b));
}

fn decision(a: Handle, s: u64, e: u64, d: u32) -> Vec<u8> {
    let mut b = context(a, s);
    b.extend(words(&[e]));
    b.extend(d.to_le_bytes());
    b.extend(0u32.to_le_bytes());
    call(a, &packet(21, &b))
}

mod correlation;
