use super::*;
#[test]
fn actual_scene_crosses_both_page_caps_without_reordering_or_lost_geometry() {
    let mut a = app();
    let id = session(&mut a);
    for _ in 0..1366 {
        draw(&mut a, id);
    }
    let mut begin = words(&[7]);
    begin.extend([2, 0, 0, 0, 0, 0, 0, 0]);
    drop(edit(&mut a, id, 6, &begin));
    let mut batch = context(&mut a, id);
    batch.extend(words(&[7]));
    batch.extend(96u32.to_le_bytes());
    batch.extend(0u32.to_le_bytes());
    for n in 1..=96u32 {
        batch.extend([0, 0, 0, 0, 1, 0, 0, 0]);
        batch.extend(words(&[u64::from(n)]));
        for v in [f64::from(n), 2.0, f64::from(n), 0.5] {
            batch.extend(v.to_le_bytes());
        }
    }
    drop(a.call(&packet(12, &batch)).unwrap());
    drop(edit(&mut a, id, 7, &words(&[7])));
    let mut query = words(&[id, 0]);
    for v in [-100.0f64, -100.0, 400.0, 400.0, 2.0] {
        query.extend(v.to_le_bytes());
    }
    let r = a.call(&packet(40, &query)).unwrap();
    let scene = u64_at(r.bytes(), 72);
    drop(r);
    let expected = a.scenes.lease(scene).unwrap().scene.clone();
    assert_eq!(expected.committed.len(), 1367);
    let mut collected = Vec::new();
    let mut first = 0u32;
    let mut last_geometry = 0;
    loop {
        let mut b = words(&[scene]);
        b.extend(first.to_le_bytes());
        b.extend(u32::MAX.to_le_bytes());
        let r = a.call(&packet(41, &b)).unwrap();
        let count = u32_at(r.bytes(), 72);
        assert!(count <= 1365);
        if count == 0 {
            break;
        }
        for chunk in r.bytes()[80..].as_chunks::<48>().0 {
            collected.push((u64_at(chunk, 0), u64_at(chunk, 8)));
            last_geometry = u64_at(chunk, 16);
            let info = a
                .call(&packet(42, &words(&[scene, last_geometry])))
                .unwrap();
            assert!(u64_at(info.bytes(), 120) >= 6);
        }
        first += count;
    }
    assert_eq!(
        collected,
        expected
            .committed
            .iter()
            .map(|s| (s.stroke_id, s.order as u64))
            .collect::<Vec<_>>()
    );
    let geometry = &expected.committed.last().unwrap().geometry;
    assert!(geometry.commands.len() > 744);
    let mut actual = Vec::new();
    let mut first = 0u32;
    loop {
        let mut b = words(&[scene, last_geometry]);
        b.extend(first.to_le_bytes());
        b.extend(u32::MAX.to_le_bytes());
        let r = a.call(&packet(43, &b)).unwrap();
        let count = u32_at(r.bytes(), 72);
        assert!(count <= 744);
        if count == 0 {
            break;
        }
        actual.extend_from_slice(&r.bytes()[80..]);
        first += count;
    }
    let mut wanted = Vec::new();
    for command in geometry.commands.iter() {
        wanted.extend(command.verb.to_le_bytes());
        wanted.extend(0u32.to_le_bytes());
        for v in command.rgba.into_iter().chain(command.points) {
            wanted.extend(v.to_le_bytes());
        }
    }
    assert_eq!(actual, wanted);
    assert_eq!(first as usize, geometry.commands.len());
    drop(expected);
    drop(a.call(&packet(44, &words(&[scene]))).unwrap());
    assert_eq!(a.scenes.bytes, 0);
    assert!(a.scenes.geometries.is_empty());
    assert!(a.scenes.roots.is_empty());
    assert!(a.scenes.geometry_ids.is_empty());
}
