use super::*;
impl EditorSession {
    pub fn execute(
        &mut self,
        context: CommandContext,
        command: EditorCommand,
    ) -> Result<CommandOutcome, EditorError> {
        let operation = command.operation();
        let fail = |kind| EditorError {
            operation,
            kind,
            context: Some(context),
        };
        self.check_context(context, command.transaction_bound())
            .map_err(fail)?;
        if command.needs_edit() && !self.capabilities.edit_document {
            return Err(fail(FailureKind::ReadOnly));
        }
        use EditorCommand::*;
        match &command {
            SetTool(_) | NewDocument | ReplaceDocument(_) => self.require_idle().map_err(fail)?,
            BeginStroke { .. } => self.require_tool(ToolId::Pen).map_err(fail)?,
            SelectAt { .. }
            | SelectCycle(_)
            | ClearSelection
            | BeginTransform
            | UpdateTransform { .. }
            | CommitTransform => self.require_tool(ToolId::Select).map_err(fail)?,
            _ => {}
        }
        // All fallible ID/counter reservations precede any document/session mutation.
        let generation = self.next_generation().map_err(fail)?;
        let starting = matches!(command, BeginStroke { .. } | BeginTransform);
        let next_interaction = if starting {
            self.next_interaction
                .checked_add(1)
                .ok_or_else(|| fail(FailureKind::ResourceLimit))?
        } else {
            self.next_interaction
        };
        let replacement = if matches!(command, NewDocument | ReplaceDocument(_)) {
            Some(DocumentSessionId::fresh().map_err(fail)?)
        } else {
            None
        };
        if matches!(command, CommitStroke(_) | CommitTransform | DeleteSelection) {
            self.editor
                .check_revision_capacity()
                .map_err(|_| fail(FailureKind::ResourceLimit))?;
        }
        let before = self.view_state();
        let invalidation = match &command {
            SetBrush(_) => RenderInvalidation::None,
            SetTool(_) | SelectAt { .. } | SelectCycle(_) | ClearSelection => {
                RenderInvalidation::Overlay
            }
            _ => RenderInvalidation::Full,
        };
        let preview_changed = matches!(command, UpdateTransform { .. });
        let result = match command {
            SetTool(tool) => {
                self.tool = tool;
                Ok(())
            }
            SetBrush(brush) => {
                self.brush = brush;
                Ok(())
            }
            SelectAt { x, y, tolerance } => self.editor.select_at(x, y, tolerance),
            SelectCycle(direction) => self.editor.select_cycle(direction),
            ClearSelection => self.editor.select_clear(),
            BeginStroke { pointer, device } => {
                self.editor
                    .begin(pointer.0, device.code(), self.brush.brush())
            }
            CommitStroke(pointer) => self.editor.commit(pointer.0),
            BeginTransform => self.editor.resize_begin(),
            UpdateTransform {
                scale,
                anchor_x,
                anchor_y,
            } => self.editor.resize_update(scale, anchor_x, anchor_y),
            CommitTransform => self.editor.resize_commit(),
            CancelInteraction => {
                self.editor.cancel();
                Ok(())
            }
            DeleteSelection => self.editor.delete_selected(),
            Undo => self.editor.undo(),
            Redo => self.editor.redo(),
            NewDocument => self.editor.clear(),
            ReplaceDocument(document) => self.editor.replace_document(document.0),
        };
        result.map_err(|e| fail(e.into()))?;
        if starting {
            self.transaction = Some(InteractionId(self.next_interaction));
        }
        if matches!(
            operation,
            Operation::CommitStroke | Operation::CommitTransform | Operation::Cancel
        ) {
            self.transaction = None;
        }
        if let Some(session) = replacement {
            self.session = session;
            self.save = super::super::storage::SaveState::default();
        }
        self.next_interaction = next_interaction;
        self.generation = generation;
        Ok(self.outcome(before, invalidation, preview_changed))
    }
}
