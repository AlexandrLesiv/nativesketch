use sketch_core::{
    Bounds,
    application::{EditorViewState, InteractionState},
};
#[derive(Debug, PartialEq, Eq)]
struct Key {
    session: u64,
    revision: u64,
    selected: u64,
    preview: u64,
}
#[derive(Default)]
pub struct SelectionCache {
    value: Option<(Key, Bounds)>,
}
impl SelectionCache {
    pub fn clear(&mut self) {
        self.value = None;
    }
    pub fn bounds(
        &mut self,
        state: EditorViewState,
        query: impl FnOnce() -> Result<Bounds, u32>,
    ) -> Result<Option<Bounds>, u32> {
        if state.document.selected == 0 {
            return Ok(None);
        }
        let key = Key {
            session: state.session.get(),
            revision: state.revision.0,
            selected: state.document.selected,
            preview: if matches!(state.interaction, InteractionState::Transforming(_)) {
                state.scene_generation.get()
            } else {
                0
            },
        };
        if let Some((cached, bounds)) = &self.value
            && *cached == key
        {
            return Ok(Some(*bounds));
        }
        let bounds = query()?;
        self.value = Some((key, bounds));
        Ok(Some(bounds))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use sketch_core::{
        Brush,
        application::{BrushSettings, Capabilities, EditorSession, ToolId},
    };
    #[test]
    fn unchanged_selected_stroke_does_not_requery_for_ink_frames() {
        let mut editor = EditorSession::new(
            Capabilities {
                edit_document: true,
                export_document: true,
            },
            ToolId::Pen,
            BrushSettings::try_from(Brush {
                width: 4.0,
                rgba: [0.0; 4],
            })
            .unwrap(),
            None,
        )
        .unwrap();
        let mut state = editor.view_state();
        state.document.selected = 5;
        let mut cache = SelectionCache::default();
        cache.bounds(state, || Ok(Bounds::default())).unwrap();
        use sketch_core::application::{
            CommandContext, DeviceKind, EditorCommand, PointerId, RequestId, SampleBatch,
        };
        let mut context = CommandContext {
            session: state.session,
            request: RequestId(1),
            interaction: None,
        };
        let outcome = editor
            .execute(
                context,
                EditorCommand::BeginStroke {
                    pointer: PointerId(1),
                    device: DeviceKind::Pen,
                },
            )
            .unwrap();
        let InteractionState::Drawing(id) = outcome.state.interaction else {
            panic!()
        };
        context.interaction = Some(id);
        for id in 1..=100 {
            let samples = [sketch_core::Input::Actual(sketch_core::Sample {
                x: id as f64,
                y: 0.0,
                timestamp: id as f64,
                pressure: None,
                id,
            })];
            editor
                .append(
                    context,
                    SampleBatch {
                        pointer: PointerId(1),
                        samples: &samples,
                    },
                )
                .unwrap();
            state = editor.view_state();
            state.document.selected = 5;
            cache
                .bounds(state, || {
                    panic!("selected stroke rescanned during actual ink input")
                })
                .unwrap();
        }
        state.interaction = InteractionState::Transforming(id);
        cache.bounds(state, || Ok(Bounds::default())).unwrap();
        state.revision.0 += 1;
        let changed = Bounds {
            width: 42.0,
            ..Default::default()
        };
        assert_eq!(cache.bounds(state, || Ok(changed)).unwrap(), Some(changed));
        cache.clear();
        assert_eq!(
            cache.bounds(state, || Ok(Bounds::default())).unwrap(),
            Some(Bounds::default())
        );
    }
}
