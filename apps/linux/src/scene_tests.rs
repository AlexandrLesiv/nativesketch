use crate::{model::Model, scene_painter};
use gtk::cairo::{Context, Format, ImageSurface};
use sketch_app::Intent;
use sketch_core::{RenderViewport, SceneCache, application::*};
fn pixels(model: &Model) -> Vec<u8> {
    let scene = model
        .owner
        .app
        .scene(
            model.owner.session,
            &mut SceneCache::default(),
            RenderViewport {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 200.0,
                margin: 2.0,
            },
        )
        .unwrap();
    let mut surface = ImageSurface::create(Format::ARgb32, 200, 200).unwrap();
    {
        let context = Context::new(&surface).unwrap();
        scene_painter::paint(&context, &scene).unwrap();
    }
    surface.data().unwrap().to_vec()
}
#[test]
fn cairo_scene_coordinates_and_transform_preview_suppress_original() {
    let mut model = Model::new();
    model.down(40.0, 50.0, None, 100, 0).unwrap();
    model.up().unwrap();
    let before = pixels(&model);
    assert!(before[(50 * 200 + 40) * 4 + 3] > 0);
    model
        .owner
        .dispatch(Intent::Action(EditorAction::CycleSelection(
            CycleDirection::Next,
        )))
        .unwrap();
    model
        .owner
        .dispatch(Intent::Edit(EditorCommand::BeginTransform))
        .unwrap();
    model
        .owner
        .dispatch(Intent::Edit(EditorCommand::UpdateTransform {
            scale: 2.0,
            anchor_x: 0.0,
            anchor_y: 0.0,
        }))
        .unwrap();
    let preview = pixels(&model);
    assert_eq!(preview[(50 * 200 + 40) * 4 + 3], 0);
    assert!(preview[(100 * 200 + 80) * 4 + 3] > 0);
    model.cancel();
    assert_eq!(pixels(&model), before);
}
