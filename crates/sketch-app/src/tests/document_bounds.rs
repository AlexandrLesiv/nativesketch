use super::*;
// RENDER-001 / ASYNC-001: the facade query is whole-document and owner checked, not scene-derived.
#[test]
fn document_bounds_remain_available_when_viewport_culls_all_strokes_and_draft_is_active() {
    let (mut app, h) = app(AppLimits::default(), true);
    assert_eq!(app.document_bounds(h).unwrap(), None);
    stroke(&mut app, h);
    let original = app.document_bounds(h).unwrap();
    assert!(original.is_some());
    let viewport = sketch_core::RenderViewport {
        x: 1e8,
        y: 1e8,
        width: 100.0,
        height: 100.0,
        margin: 0.0,
    };
    let mut cache = sketch_core::SceneCache::default();
    assert!(
        app.scene(h, &mut cache, viewport)
            .unwrap()
            .committed
            .is_empty()
    );
    draft(&mut app, h);
    let before = app.projection(h).unwrap();
    assert_eq!(app.document_bounds(h).unwrap(), original);
    assert_eq!(app.projection(h).unwrap(), before);
    let (other, foreign) = super::app(AppLimits::default(), false);
    assert!(app.document_bounds(foreign).is_err());
    assert!(other.document_bounds(h).is_err());
}
#[test]
fn readonly_source_replacement_updates_document_bounds_and_detached_handle_is_rejected() {
    let (mut app, h) = app(AppLimits::default(), false);
    let outcome = app
        .begin_view_source(h, ctx(&app, h), 1, resource(1))
        .unwrap();
    let Some(HostEffect::WorkQueued(op)) = outcome.effect else {
        panic!()
    };
    let prepared = read(&mut app).prepare(FIXTURE.to_vec());
    complete(&mut app, prepared);
    app.apply_ready(h, op).unwrap();
    retire(&mut app);
    assert!(app.document_bounds(h).unwrap().is_some());
    app.detach_session(h).unwrap();
    assert!(app.document_bounds(h).is_err());
    retire(&mut app);
}
