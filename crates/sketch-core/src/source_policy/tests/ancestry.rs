use super::*;
#[test]
fn drafts_require_known_parent_and_do_not_silently_downgrade() {
    let base = version("base", "", 7, Kind::Published, A);
    let draft = version("draft", "base", 7, Kind::Draft, B);
    let child = version("child", "draft", 7, Kind::Draft, A);
    let published = version("published", "draft", 8, Kind::Published, A);
    assert_eq!(compare(Some(&base), &draft), Ok(Decision::Accept));
    assert_eq!(compare(Some(&draft), &base), Ok(Decision::Stale));
    assert_eq!(compare(Some(&draft), &child), Ok(Decision::Accept));
    assert_eq!(compare(Some(&child), &draft), Ok(Decision::Stale));
    assert_eq!(compare(Some(&draft), &published), Ok(Decision::Accept));
    assert_eq!(compare(Some(&published), &draft), Ok(Decision::Stale));
    let unrelated = version("remote", "unknown", 8, Kind::Published, A);
    assert_eq!(compare(Some(&draft), &unrelated), Ok(Decision::Conflict));
    let branch = version("branch", "base", 7, Kind::Draft, A);
    assert_eq!(compare(Some(&draft), &branch), Ok(Decision::Conflict));
    let skipped = version("grandchild", "child", 7, Kind::Draft, A);
    assert_eq!(compare(Some(&draft), &skipped), Ok(Decision::Conflict));
    let wrong_base = Version {
        server_generation: 8,
        ..child
    };
    assert_eq!(compare(Some(&draft), &wrong_base), Ok(Decision::Conflict));
    let phone_only = version("phone", "", 0, Kind::Draft, B);
    assert_eq!(
        compare(Some(&phone_only), &unrelated),
        Ok(Decision::Conflict)
    );
    let identical = Version {
        content_sha256: B,
        ..unrelated
    };
    assert_eq!(
        compare(Some(&phone_only), &identical),
        Ok(Decision::Conflict)
    );
    // Decisions never overwrite the caller's immutable metadata.
    assert_eq!(phone_only.kind, Kind::Draft);
    assert_eq!(phone_only.server_generation, 0);
}
