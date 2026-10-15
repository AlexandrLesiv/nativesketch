use super::*;
impl EditorSession {
    pub(crate) fn validate_context(&self, context: CommandContext) -> Result<(), EditorError> {
        self.check_context(context, false)
            .map_err(|kind| EditorError {
                operation: Operation::Query,
                kind,
                context: Some(context),
            })
    }
    pub(crate) fn inherit_settings(&mut self, tool: ToolId, brush: BrushSettings) {
        self.tool = tool;
        self.brush = brush;
    }
}
