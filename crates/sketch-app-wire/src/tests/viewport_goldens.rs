use super::viewport_preparation::{leased, page, prepare};
use super::*;
#[test]
fn viewport_requests_have_exact_bytes_and_reject_every_truncation_and_tail() {
    let expected = include_bytes!("../../tests/fixtures/viewport-request.bin");
    let body = [
        words(&[0xf123456789abcd00]),
        [0u32, 0, 100, 100]
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .collect(),
    ]
    .concat();
    assert_eq!(packet(45, &body), expected);
    for (op, body) in [
        (45, body),
        (46, [words(&[0xf123456789abcd01]), vec![0; 8]].concat()),
        (47, words(&[0xf123456789abcd01])),
    ] {
        let bytes = packet(op, &body);
        decode(&bytes).unwrap();
        for end in 0..bytes.len() {
            assert!(decode(&bytes[..end]).is_err());
        }
        let mut tail = body;
        tail.push(0);
        assert!(decode(&packet(op, &tail)).is_err());
    }
}
#[test]
fn viewport_reply_and_whole_contour_page_match_literal_goldens() {
    let mut a = app();
    let id = session(&mut a);
    let scene = leased(&mut a, id);
    let reply = prepare(&mut a, scene, 0, 0).unwrap();
    let prepared = u64_at(reply.bytes(), 80);
    let mut actual = reply.bytes().to_vec();
    drop(reply);
    for (offset, value) in [
        (80, 0xf123456789abcd01u64),
        (88, 0xf123456789abcd00),
        (96, 0xf123456789abcd02),
    ] {
        actual[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }
    assert_eq!(
        actual,
        include_bytes!("../../tests/fixtures/viewport-reply.bin")
    );
    assert_eq!(
        page(&mut a, prepared, 0, 5).unwrap().bytes(),
        include_bytes!("../../tests/fixtures/viewport-paths.bin")
    );
}
