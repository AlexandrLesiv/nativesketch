use super::*;
#[test]
fn identity_and_immutable_revision_are_checked_before_comparison() {
    let a = version("r1", "", 1, Kind::Published, A);
    let mut b = a;
    assert_eq!(compare(None, &a), Ok(Decision::Accept));
    assert_eq!(compare(Some(&a), &b), Ok(Decision::Duplicate));
    b.namespace_id = "different-account/library";
    assert_eq!(compare(Some(&a), &b), Ok(Decision::WrongIdentity));
    b = a;
    b.document_id = "different-drawing";
    assert_eq!(compare(Some(&a), &b), Ok(Decision::WrongIdentity));
    b = a;
    b.content_sha256 = B;
    assert_eq!(compare(Some(&a), &b), Err(Error::InvalidArgument));
    b = a;
    b.server_generation = 2;
    assert_eq!(compare(Some(&a), &b), Err(Error::InvalidArgument));
    b = a;
    b.parent_revision_id = "different-parent";
    assert_eq!(compare(Some(&a), &b), Err(Error::InvalidArgument));
    b = a;
    b.kind = Kind::Draft;
    assert_eq!(compare(Some(&a), &b), Err(Error::InvalidArgument));
}
#[test]
fn published_order_uses_generation_without_clock_or_parent_assumptions() {
    let a = version("r1", "", 1, Kind::Published, A);
    let mut b = version("r3", "r2", 3, Kind::Published, B);
    assert_eq!(compare(Some(&a), &b), Ok(Decision::Accept));
    assert_eq!(compare(Some(&b), &a), Ok(Decision::Stale));
    b.server_generation = 1;
    assert_eq!(compare(Some(&a), &b), Ok(Decision::Conflict));
    b.content_sha256 = A;
    assert_eq!(compare(Some(&a), &b), Ok(Decision::Conflict));
    b.server_generation = 3;
    assert_eq!(compare(Some(&a), &b), Ok(Decision::Accept));
}
#[test]
fn malformed_metadata_and_cycles_are_rejected() {
    let valid = version("r1", "", 1, Kind::Published, A);
    let too_long = "x".repeat(257);
    for bad in [
        Version {
            namespace_id: "",
            ..valid
        },
        Version {
            document_id: "a\0b",
            ..valid
        },
        Version {
            revision_id: &too_long,
            ..valid
        },
        Version {
            parent_revision_id: "r1",
            ..valid
        },
        Version {
            content_sha256: "abcdef",
            ..valid
        },
        Version {
            content_sha256: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            ..valid
        },
        Version {
            server_generation: 0,
            ..valid
        },
    ] {
        assert_eq!(compare(None, &bad), Err(Error::InvalidArgument));
        assert_eq!(compare(Some(&bad), &valid), Err(Error::InvalidArgument));
    }
    let a = version("a", "b", 0, Kind::Draft, A);
    let b = version("b", "a", 0, Kind::Draft, B);
    assert_eq!(compare(Some(&a), &b), Err(Error::InvalidArgument));
}
