use sketch_core::{
    Brush, Input, RenderScene, Sample, SceneCache,
    application::{
        BrushSettings, Capabilities, CommandContext, DeviceKind, EditorCommand, EditorSession,
        InteractionState, PointerId, RequestId, SampleBatch, ToolId,
    },
};
use sketch_renderer::RenderRequest;
use std::time::Instant;
pub struct Fixture {
    editor: EditorSession,
    cache: SceneCache,
    serial: u64,
}
impl Fixture {
    pub fn new() -> Self {
        let editor = EditorSession::new(
            Capabilities {
                edit_document: true,
                export_document: true,
            },
            ToolId::Pen,
            BrushSettings::try_from(Brush {
                width: 5.0,
                rgba: [0.15, 0.25, 0.8, 0.6],
            })
            .unwrap(),
            None,
        )
        .unwrap();
        let mut value = Self {
            editor,
            cache: SceneCache::default(),
            serial: 0,
        };
        for row in 0..128 {
            value.command(EditorCommand::BeginStroke {
                pointer: PointerId(1),
                device: DeviceKind::Pen,
            });
            let inputs = (1..=32)
                .map(|id| {
                    Input::Actual(Sample {
                        x: 20.0 + (row % 8) as f64 * 120.0 + id as f64 * 2.0,
                        y: 20.0 + (row / 8) as f64 * 44.0 + ((id * 7) % 19) as f64,
                        timestamp: id as f64 / 240.0,
                        pressure: Some(0.2 + id as f64 / 50.0),
                        id,
                    })
                })
                .collect::<Vec<_>>();
            value.input(&inputs);
            value.command(EditorCommand::CommitStroke(PointerId(1)));
        }
        value
    }
    fn context(&mut self) -> CommandContext {
        self.serial += 1;
        let state = self.editor.view_state();
        let interaction = match state.interaction {
            InteractionState::Idle => None,
            InteractionState::Drawing(id) | InteractionState::Transforming(id) => Some(id),
        };
        CommandContext {
            session: state.session,
            request: RequestId(self.serial),
            interaction,
        }
    }
    pub fn command(&mut self, command: EditorCommand) -> u64 {
        let context = self.context();
        let start = Instant::now();
        self.editor.execute(context, command).unwrap();
        start.elapsed().as_nanos() as u64
    }
    fn input(&mut self, samples: &[Input]) -> u64 {
        let context = self.context();
        let start = Instant::now();
        self.editor
            .append(
                context,
                SampleBatch {
                    pointer: PointerId(1),
                    samples,
                },
            )
            .unwrap();
        start.elapsed().as_nanos() as u64
    }
    pub fn append(&mut self, id: u64) -> u64 {
        let sample = |id: u64| Sample {
            x: 100.0 + id as f64 * 3.0,
            y: 100.0 + ((id * 11) % 31) as f64,
            timestamp: id as f64 / 240.0,
            pressure: Some(0.5),
            id,
        };
        self.input(&[Input::Actual(sample(id)), Input::Predicted(sample(id + 1))])
    }
    pub fn prepare(&mut self) -> (RenderScene, RenderRequest, u64) {
        let state = self.editor.view_state();
        let request = RenderRequest {
            struct_size: size_of::<RenderRequest>() as u32,
            session: state.session.get(),
            revision: state.revision.0,
            scene_generation: state.scene_generation.get(),
            surface_generation: 1,
            logical_width: 1024.0,
            logical_height: 768.0,
            pixel_width: 1024,
            pixel_height: 768,
            zoom: 1.0,
            ..Default::default()
        };
        let start = Instant::now();
        let scene = self
            .editor
            .render_scene(&mut self.cache, request.viewport())
            .unwrap();
        (scene, request, start.elapsed().as_nanos() as u64)
    }
}
