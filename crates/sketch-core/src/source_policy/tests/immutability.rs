use super::*;
#[test]
fn same_content_published_advance_retains_generation_against_delayed_rollback() {
    let r1 = version("r1", "", 1, Kind::Published, A);
    let r2 = version("r2", "r1", 2, Kind::Published, B);
    let r3 = version("r3", "r2", 3, Kind::Published, A);
    let mut cached = r1;
    // A revert on the server changes revision without changing r1's bytes.
    assert_eq!(compare(Some(&cached), &r3), Ok(Decision::Accept));
    cached = r3;
    assert_eq!(compare(Some(&cached), &r2), Ok(Decision::Stale));
    assert_eq!(compare(Some(&cached), &r1), Ok(Decision::Stale));
    assert_eq!(compare(Some(&cached), &r3), Ok(Decision::Duplicate));
}
#[test]
fn same_content_draft_advance_preserves_parent_chain() {
    let r1 = version("r1", "", 1, Kind::Published, A);
    let d1 = version("d1", "r1", 1, Kind::Draft, A);
    let d2 = version("d2", "d1", 1, Kind::Draft, B);
    let mut cached = r1;
    assert_eq!(compare(Some(&cached), &d1), Ok(Decision::Accept));
    cached = d1;
    assert_eq!(compare(Some(&cached), &d2), Ok(Decision::Accept));
    cached = d2;
    assert_eq!(compare(Some(&cached), &d1), Ok(Decision::Stale));
    // Publishing unchanged bytes must likewise advance provenance.
    let published = version("r2", "d2", 2, Kind::Published, B);
    assert_eq!(compare(Some(&cached), &published), Ok(Decision::Accept));
    cached = published;
    assert_eq!(compare(Some(&cached), &d2), Ok(Decision::Stale));
    let unrelated = version("unknown-draft", "", 2, Kind::Draft, B);
    assert_eq!(compare(Some(&cached), &unrelated), Ok(Decision::Conflict));
}
