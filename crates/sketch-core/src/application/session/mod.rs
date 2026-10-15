mod dispatch;
mod guard;
mod projections;
mod routing;
mod storage;
use super::*;
/// One owner performs mutations through exclusive references. This type exposes no
/// mutable legacy editor or document. Worker results must return to this owner.
#[derive(Debug)]
pub struct EditorSession {
    editor: Editor,
    capabilities: Capabilities,
    tool: ToolId,
    brush: BrushSettings,
    session: DocumentSessionId,
    transaction: Option<InteractionId>,
    next_interaction: u64,
    generation: SceneGeneration,
    save: super::storage::SaveState,
}
impl EditorSession {
    /// Hosts supply their own defaults. An initial validated viewer document is allowed;
    /// changing viewer sources later requires task 022's explicit source lifecycle.
    pub fn new(
        capabilities: Capabilities,
        tool: ToolId,
        brush: BrushSettings,
        initial: Option<ValidatedDocument>,
    ) -> Result<Self, EditorError> {
        let fail = |kind| EditorError {
            operation: Operation::Construct,
            kind,
            context: None,
        };
        if !capabilities.edit_document && tool != ToolId::Select {
            return Err(fail(FailureKind::ReadOnly));
        }
        let session = DocumentSessionId::fresh().map_err(fail)?;
        let mut editor = Editor::new();
        if let Some(document) = initial {
            editor
                .replace_document(document.0)
                .map_err(|e| fail(e.into()))?;
        }
        Ok(Self {
            editor,
            capabilities,
            tool,
            brush,
            session,
            transaction: None,
            next_interaction: 1,
            generation: SceneGeneration(0),
            save: super::storage::SaveState::default(),
        })
    }
    pub fn append(
        &mut self,
        context: CommandContext,
        batch: SampleBatch<'_>,
    ) -> Result<CommandOutcome, EditorError> {
        let fail = |kind| EditorError {
            operation: Operation::Append,
            kind,
            context: Some(context),
        };
        self.check_context(context, true).map_err(fail)?;
        if !self.capabilities.edit_document {
            return Err(fail(FailureKind::ReadOnly));
        }
        let generation = self.next_generation().map_err(fail)?;
        let before = self.view_state();
        self.editor
            .append(batch.pointer.0, batch.samples)
            .map_err(|e| fail(e.into()))?;
        self.generation = generation;
        Ok(self.outcome(before, RenderInvalidation::Full, true))
    }
}

#[cfg(test)]
mod tests;
