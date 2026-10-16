use crate::{Compositor, RenderRequest, RenderStats};
use sketch_core::{
    Bounds, Brush, PathCommand, RenderScene, SceneStroke, StrokeGeometry, Transform,
    application::{BrushSettings, Capabilities, EditorSession, SceneKey, ToolId},
};
use skia_safe::{AlphaType, ColorSpace, ColorType, ImageInfo, surfaces};
use std::sync::Arc;
pub fn rectangle(id: u64, order: usize, x: f64, color: [f64; 4]) -> SceneStroke {
    let commands = [
        (0, [x, 4.0]),
        (1, [x + 20.0, 4.0]),
        (1, [x + 20.0, 24.0]),
        (1, [x, 24.0]),
        (3, [0.0, 0.0]),
    ]
    .into_iter()
    .map(|(verb, p)| PathCommand {
        stroke_id: id,
        rgba: color,
        verb,
        points: [p[0], p[1], 0.0, 0.0, 0.0, 0.0],
    })
    .collect::<Vec<_>>();
    SceneStroke {
        stroke_id: id,
        order,
        transform: Transform::default(),
        geometry: Arc::new(StrokeGeometry {
            commands: commands.into(),
            bounds: Bounds {
                x,
                y: 4.0,
                width: 20.0,
                height: 20.0,
            },
            origin: [x, 4.0],
        }),
    }
}
pub fn scene(strokes: Vec<SceneStroke>) -> (RenderScene, RenderRequest) {
    let editor = EditorSession::new(
        Capabilities {
            edit_document: true,
            export_document: true,
        },
        ToolId::Pen,
        BrushSettings::try_from(Brush {
            width: 4.0,
            rgba: [0.0, 0.0, 0.0, 1.0],
        })
        .unwrap(),
        None,
    )
    .unwrap();
    let state = editor.view_state();
    let key = SceneKey {
        session: state.session,
        revision: state.revision,
        generation: state.scene_generation,
    };
    let request = RenderRequest {
        struct_size: size_of::<RenderRequest>() as u32,
        session: key.session.get(),
        revision: key.revision.0,
        scene_generation: key.generation.get(),
        surface_generation: 1,
        logical_width: 64.0,
        logical_height: 64.0,
        pixel_width: 64,
        pixel_height: 64,
        zoom: 1.0,
        selection_rgba: [0.0, 0.4, 1.0, 1.0],
        focus_rgba: [0.2, 0.4, 1.0, 1.0],
        ..Default::default()
    };
    (
        RenderScene {
            key,
            committed: strokes.into(),
            live: None,
            transform_preview: None,
        },
        request,
    )
}
pub fn render(
    compositor: &mut Compositor,
    scene: &RenderScene,
    request: &RenderRequest,
    cached: bool,
) -> (Vec<u8>, RenderStats) {
    let info = ImageInfo::new(
        (request.pixel_width as i32, request.pixel_height as i32),
        ColorType::RGBA8888,
        AlphaType::Premul,
        ColorSpace::new_srgb(),
    );
    let mut surface = surfaces::raster(&info, None, None).unwrap();
    let stats = compositor
        .render(&mut surface, scene, request, None, cached)
        .unwrap();
    let mut pixels = vec![0; request.pixel_width as usize * request.pixel_height as usize * 4];
    assert!(surface.read_pixels(&info, &mut pixels, request.pixel_width as usize * 4, (0, 0)));
    (pixels, stats)
}
pub fn pixel(pixels: &[u8], x: usize, y: usize) -> &[u8] {
    &pixels[(y * 64 + x) * 4..(y * 64 + x + 1) * 4]
}
pub fn equivalent(a: &[u8], b: &[u8], name: &str) {
    if a != b {
        let root = std::env::temp_dir().join("native-sketch-skia-diffs");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join(format!("{name}-cached.rgba")), a).unwrap();
        std::fs::write(root.join(format!("{name}-reference.rgba")), b).unwrap();
        let diff = a
            .iter()
            .zip(b)
            .map(|(a, b)| a.abs_diff(*b))
            .collect::<Vec<_>>();
        std::fs::write(root.join(format!("{name}-diff.rgba")), diff).unwrap();
        panic!(
            "SKIA-VISUAL-001 exact RGBA mismatch; artifacts {}",
            root.display()
        );
    }
}
