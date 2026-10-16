use crate::*;
mod cancellation;
mod codec;
mod errors;
mod owner;
mod work;
fn fixtures() -> Vec<(String, Vec<u8>)> {
    include_str!("../../tests/fixtures/v2.json")
        .lines()
        .filter_map(|line| {
            let (name, hex) = line.trim().trim_start_matches('"').split_once("\": \"")?;
            let hex = hex.trim_end_matches(',').trim_end_matches('"');
            Some((
                name.to_string(),
                (0..hex.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                    .collect(),
            ))
        })
        .collect()
}
fn fixture(name: &str) -> Vec<u8> {
    fixtures().into_iter().find(|(n, _)| n == name).unwrap().1
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
fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}
fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
fn app() -> WireApp {
    WireApp::bootstrap(&fixture("bootstrap"), TransportLimits::default()).unwrap()
}
fn session(app: &mut WireApp) -> u64 {
    let r = app.call(&fixture("create.cap3")).unwrap();
    u64_at(r.bytes(), 72)
}
fn context(app: &mut WireApp, id: u64) -> Vec<u8> {
    let r = app.call(&packet(3, &words(&[id]))).unwrap();
    words(&[id, u64_at(r.bytes(), 80), u64_at(r.bytes(), 104)])
}
fn edit(app: &mut WireApp, id: u64, tag: u32, tail: &[u8]) -> Reply {
    let mut b = context(app, id);
    b.extend(tag.to_le_bytes());
    b.extend(0u32.to_le_bytes());
    b.extend(tail);
    app.call(&packet(10, &b)).unwrap()
}
fn file(app: &mut WireApp, id: u64, tag: u32) -> Reply {
    let mut b = context(app, id);
    b.extend(tag.to_le_bytes());
    b.extend(0u32.to_le_bytes());
    app.call(&packet(20, &b)).unwrap()
}
fn supply(
    app: &mut WireApp,
    id: u64,
    effect: u64,
    resource: u64,
    canonical: u64,
    access: u32,
) -> Reply {
    let mut b = context(app, id);
    b.extend(words(&[effect]));
    b.extend(1u32.to_le_bytes());
    b.extend(access.to_le_bytes());
    b.extend(words(&[resource, canonical, 1]));
    app.call(&packet(22, &b)).unwrap()
}
fn draw(app: &mut WireApp, id: u64) {
    let mut b = words(&[7]);
    b.extend(2u32.to_le_bytes());
    b.extend(0u32.to_le_bytes());
    drop(edit(app, id, 6, &b));
    let sample = fixture("sample.0.1");
    let mut b = context(app, id);
    b.extend_from_slice(&sample[48..]);
    drop(app.call(&packet(12, &b)).unwrap());
    drop(edit(app, id, 7, &words(&[7])));
}
fn queued(app: &mut WireApp, id: u64, kind: u32, resource: u64, access: u32) -> u64 {
    let r = file(app, id, kind);
    let effect = u64_at(r.bytes(), 232 + 8);
    drop(r);
    let r = supply(app, id, effect, resource, 1, access);
    u64_at(r.bytes(), 248)
}
fn take(app: &mut WireApp) -> WireWork {
    app.take_work(&packet(30, &[])).unwrap().1.unwrap()
}
fn finish(app: &mut WireApp, mut work: WireWork) {
    drop(work.call(&packet(101, &[]), &mut None).unwrap());
    let mut result = work
        .call(&packet(104, &[1, 0, 0, 0, 3, 0, 0, 0]), &mut None)
        .unwrap()
        .result;
    drop(app.complete(&packet(31, &[]), &mut result).unwrap());
    assert!(result.is_none());
}
fn drain(app: &mut WireApp) {
    loop {
        let (r, w) = app.take_work(&packet(30, &[])).unwrap();
        drop(r);
        let Some(mut w) = w else {
            break;
        };
        let mut result = w.call(&packet(106, &[]), &mut None).unwrap().result;
        drop(app.complete(&packet(31, &[]), &mut result).unwrap());
    }
}
mod diagnostic_goldens;
mod effect_goldens;
mod enum_records;
mod failure_goldens;
mod response_goldens;
mod scene_goldens;
mod scenes;
mod stale_load;
mod tag_goldens;
mod transfer_capacity;
