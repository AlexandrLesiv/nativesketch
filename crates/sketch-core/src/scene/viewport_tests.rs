use super::*;
use crate::{Bounds, Transform};
use std::sync::Arc;
fn stroke(end: f64) -> SceneStroke {
    let commands = [
        (0, [0.0, 18.0]),
        (1, [end, 18.0]),
        (1, [end, 22.0]),
        (1, [0.0, 22.0]),
        (3, [0.0, 0.0]),
    ]
    .into_iter()
    .map(|(verb, p)| PathCommand {
        stroke_id: 7,
        rgba: [1.0, 0.0, 0.0, 0.5],
        verb,
        points: [p[0], p[1], 0.0, 0.0, 0.0, 0.0],
    })
    .collect::<Vec<_>>();
    SceneStroke {
        stroke_id: 7,
        order: 0,
        transform: Transform::default(),
        geometry: Arc::new(StrokeGeometry {
            commands: commands.into(),
            origin: [end.min(0.0), 18.0],
            bounds: Bounds {
                x: end.min(0.0),
                y: 18.0,
                width: end.abs(),
                height: 4.0,
            },
        }),
    }
}
fn viewport(x: f64) -> PixelViewport {
    PixelViewport {
        document: RenderViewport {
            x,
            y: 0.0,
            width: 100.0,
            height: 160.0,
            margin: 2.0,
        },
        pixel_width: 133,
        pixel_height: 219,
    }
}
#[test]
fn fractional_backing_and_transforms_preserve_both_giant_endpoints() {
    for sign in [1.0, -1.0] {
        let mut s = stroke(sign * 999_999_010.0);
        s.transform = Transform {
            scale: 2.0,
            tx: 123.25,
            ty: 2.5,
        };
        let end = sign * 999_999_010.0 * 2.0 + 123.25;
        let v = viewport(end - 10.0);
        let PreparedViewportPath::PixelCommands(commands) =
            prepare_viewport_path(&s, v, PreparationLimits::default()).unwrap()
        else {
            panic!("giant stroke requires preparation")
        };
        assert!(
            commands
                .iter()
                .any(|p| p.verb < 2 && (p.points[0] - 13.3).abs() < 1e-8)
        );
        assert!(
            commands
                .iter()
                .filter(|p| p.verb < 2)
                .all(|p| (-2.0..=135.0).contains(&p.points[0]))
        );
        assert!(commands.iter().all(|p| p.rgba == [1.0, 0.0, 0.0, 0.5]));
        assert!(
            commands
                .iter()
                .any(|p| p.verb < 2 && (p.points[1] - 38.5 * 219.0 / 160.0).abs() < 1e-8)
        );
        assert_eq!(s.geometry.commands[1].points[0], sign * 999_999_010.0);
    }
}
#[test]
fn normal_local_geometry_needs_no_owned_output() {
    assert!(matches!(
        prepare_viewport_path(&stroke(10.0), viewport(0.0), PreparationLimits::default()).unwrap(),
        PreparedViewportPath::LocalGeometry
    ));
}
#[test]
fn preparation_rejects_memory_points_work_depth_and_nonfinite_inputs() {
    let s = stroke(999_999_010.0);
    let v = viewport(999_999_000.0);
    for limits in [
        PreparationLimits {
            bytes: 256,
            ..Default::default()
        },
        PreparationLimits {
            points: 0,
            ..Default::default()
        },
        PreparationLimits {
            work: 0,
            ..Default::default()
        },
    ] {
        assert_eq!(
            prepare_viewport_path(&s, v, limits).unwrap_err(),
            SceneError::ResourceLimit
        );
    }
    let mut invalid = v;
    invalid.document.x = f64::NAN;
    assert_eq!(
        prepare_viewport_path(&s, invalid, PreparationLimits::default()).unwrap_err(),
        SceneError::InvalidViewport
    );
    invalid = v;
    invalid.pixel_width = 0;
    assert_eq!(
        prepare_viewport_path(&s, invalid, PreparationLimits::default()).unwrap_err(),
        SceneError::InvalidViewport
    );
    let mut s = s;
    let g = Arc::get_mut(&mut s.geometry).unwrap();
    let commands = Arc::make_mut(&mut g.commands);
    commands[1].verb = 2;
    commands[1].points = [-1e9, 1e9, 1e9, 1e9, 1e9, -1e9];
    commands[0].points = [-1e9, -1e9, 0.0, 0.0, 0.0, 0.0];
    assert_eq!(
        prepare_viewport_path(
            &s,
            viewport(0.0),
            PreparationLimits {
                depth: 0,
                ..Default::default()
            }
        )
        .unwrap_err(),
        SceneError::ResourceLimit
    );
    Arc::make_mut(&mut Arc::get_mut(&mut s.geometry).unwrap().commands)[1].points[0] =
        f64::INFINITY;
    assert_eq!(
        prepare_viewport_path(&s, v, PreparationLimits::default()).unwrap_err(),
        SceneError::InvalidViewport
    );
}
#[test]
fn multiple_contours_keep_close_and_single_fill_color() {
    let mut s = stroke(1e9);
    let g = Arc::get_mut(&mut s.geometry).unwrap();
    g.commands = [g.commands.as_ref(), g.commands.as_ref()].concat().into();
    let PreparedViewportPath::PixelCommands(commands) =
        prepare_viewport_path(&s, viewport(1e9 - 10.0), PreparationLimits::default()).unwrap()
    else {
        panic!()
    };
    assert_eq!(commands.iter().filter(|p| p.verb == 0).count(), 2);
    assert_eq!(commands.iter().filter(|p| p.verb == 3).count(), 2);
    assert!(commands.iter().all(|p| p.rgba[3] == 0.5));
}
