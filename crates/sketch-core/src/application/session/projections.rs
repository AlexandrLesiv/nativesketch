use super::*;
impl EditorSession {
    /// O(1): neither sample traversal, document cloning, geometry nor encoding.
    pub fn view_state(&self) -> EditorViewState {
        let document = self.editor.summary();
        let interaction = match (document.interaction, self.transaction) {
            (1, Some(id)) => InteractionState::Drawing(id),
            (2, Some(id)) => InteractionState::Transforming(id),
            _ => InteractionState::Idle,
        };
        EditorViewState {
            session: self.session,
            revision: Revision(document.revision),
            scene_generation: self.generation,
            tool: self.tool,
            brush: self.brush,
            capabilities: self.capabilities,
            interaction,
            document,
        }
    }
    /// Explicit geometry/copy queries may be O(document size); never called by execute/append.
    pub fn query(&self, query: EditorQuery) -> Result<QueryResult, EditorError> {
        let state = self.view_state();
        Ok(match query {
            EditorQuery::ViewState => QueryResult::ViewState(state),
            EditorQuery::SelectedBounds => QueryResult::SelectedBounds(self.editor.state().bounds),
            EditorQuery::DocumentBounds => QueryResult::DocumentBounds(self.document_bounds()),
            EditorQuery::Scene => QueryResult::Scene(SceneSnapshot {
                key: SceneKey {
                    session: state.session,
                    revision: state.revision,
                    generation: self.generation,
                },
                commands: self.editor.render(),
            }),
            EditorQuery::CommittedSnapshot => QueryResult::Document(DocumentSnapshot {
                document: self.editor.document().clone(),
                session: state.session,
                revision: state.revision,
            }),
        })
    }
    pub(super) fn outcome(
        &self,
        before: EditorViewState,
        invalidation: RenderInvalidation,
        preview_changed: bool,
    ) -> CommandOutcome {
        let state = self.view_state();
        CommandOutcome {
            changes: StateChanges {
                document: state.revision != before.revision,
                session_replaced: state.session != before.session,
                settings: state.tool != before.tool || state.brush != before.brush,
                selection: state.document.selected != before.document.selected,
                interaction: preview_changed || state.interaction != before.interaction,
            },
            state,
            invalidation,
        }
    }
}
