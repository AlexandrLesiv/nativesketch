use super::support::*;

#[test]
fn eviction_clear_and_zero_retention_preserve_reference_geometry_across_viewports() {
    let mut editor = editor();
    for index in 0..40 {
        stroke(&mut editor, index as f64 * 10.0);
    }
    let mut cache = SceneCache::new(SceneLimits {
        retained_bytes: 2048,
        prepared_bytes: 1024 * 1024,
    });
    let original = editor.render_scene(&mut cache, viewport()).unwrap();
    assert!(cache.stats().retained_bytes <= 2048);
    assert_eq!(cache.stats().retained_entries, 1);
    reference(&editor, &original);
    let mut uncached = SceneCache::new(SceneLimits {
        retained_bytes: 0,
        prepared_bytes: 1024 * 1024,
    });
    for index in 0..20 {
        let view = RenderViewport {
            x: index as f64 * 7.0,
            width: 70.0,
            ..viewport()
        };
        let warm = editor.render_scene(&mut cache, view).unwrap();
        uncached.clear();
        let cold = editor.render_scene(&mut uncached, view).unwrap();
        equivalent(&commands(&warm), &commands(&cold));
        assert!(cache.stats().retained_bytes <= 2048);
        assert_eq!(uncached.stats().retained_bytes, 0);
    }
    cache.clear();
    assert_eq!(cache.stats().retained_entries, 0);
    reference(&editor, &original); // Issued immutable snapshots survive eviction and reset.
    equivalent(
        &commands(&original),
        &commands(&editor.render_scene(&mut cache, viewport()).unwrap()),
    );
}

#[test]
fn budget_failure_is_explicit_and_does_not_mutate_document_or_poison_future_queries() {
    let mut editor = editor();
    stroke(&mut editor, 10.0);
    let saved = bytes(&editor);
    let before = editor.view_state();
    let mut tiny = SceneCache::new(SceneLimits {
        retained_bytes: 0,
        prepared_bytes: 1000,
    });
    assert!(matches!(
        editor.render_scene(&mut tiny, viewport()),
        Err(SceneError::ResourceLimit)
    ));
    assert_eq!(bytes(&editor), saved);
    assert_eq!(editor.view_state(), before);
    let offscreen = RenderViewport {
        x: 500.0,
        width: 100.0,
        ..viewport()
    };
    assert!(
        editor
            .render_scene(&mut tiny, offscreen)
            .unwrap()
            .committed
            .is_empty()
    );
    assert!(matches!(
        editor.render_scene(&mut tiny, viewport()),
        Err(SceneError::ResourceLimit)
    ));
    reference(
        &editor,
        &editor
            .render_scene(&mut SceneCache::default(), viewport())
            .unwrap(),
    );
}

#[test]
fn invalid_viewport_is_rejected_before_cache_or_document_changes() {
    let mut editor = editor();
    stroke(&mut editor, 10.0);
    let mut cache = SceneCache::default();
    let original = editor.render_scene(&mut cache, viewport()).unwrap();
    let stats = cache.stats();
    for view in [
        RenderViewport {
            x: f64::NAN,
            ..viewport()
        },
        RenderViewport {
            width: 0.0,
            ..viewport()
        },
        RenderViewport {
            margin: -1.0,
            ..viewport()
        },
        RenderViewport {
            x: f64::MAX,
            width: f64::MAX,
            ..viewport()
        },
    ] {
        assert!(matches!(
            editor.render_scene(&mut cache, view),
            Err(SceneError::InvalidViewport)
        ));
        assert_eq!(cache.stats(), stats);
    }
    let after = editor.render_scene(&mut cache, viewport()).unwrap();
    assert!(Arc::ptr_eq(&original.committed, &after.committed));
}
