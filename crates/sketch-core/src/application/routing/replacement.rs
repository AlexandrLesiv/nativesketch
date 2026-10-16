use super::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplacementSource {
    New,
    Document(DestinationId),
    /// Opaque source identity, deliberately never bound as a save destination.
    ViewSource(DestinationId),
    Recovery(RecoverySourceId),
}
/// Issued immutable worker input; non-Clone and contains no live editor borrow.
#[derive(Debug)]
pub struct ReplacementTicket {
    pub(super) token: EffectToken,
    pub(super) revision: Revision,
    pub(super) source: ReplacementSource,
    pub(super) capabilities: Capabilities,
    pub(super) tool: ToolId,
    pub(super) brush: BrushSettings,
}
#[derive(Debug)]
pub struct PreparedReplacement {
    pub(super) token: EffectToken,
    pub(super) revision: Revision,
    pub(super) editor: Option<EditorSession>,
}
impl PreparedReplacement {
    pub fn is_consumed(&self) -> bool {
        self.editor.is_none()
    }
}
#[derive(Debug)]
pub struct ReplacementFailure {
    pub(super) token: EffectToken,
    pub(super) failure: ApplicationFailure,
}
impl ReplacementFailure {
    pub fn failure(&self) -> ApplicationFailure {
        self.failure
    }
}
impl ReplacementTicket {
    pub fn token(&self) -> EffectToken {
        self.token
    }
    pub fn source(&self) -> ReplacementSource {
        self.source
    }
    pub fn revision(&self) -> Revision {
        self.revision
    }
    pub fn fail(self, failure: StorageFailure) -> ReplacementFailure {
        ReplacementFailure {
            token: self.token,
            failure: ApplicationFailure::Write(failure),
        }
    }
    /// Worker-only preparation: validates/constructs and binds before any live state replacement.
    pub fn prepare(
        self,
        document: Option<ValidatedDocument>,
    ) -> Result<PreparedReplacement, ReplacementFailure> {
        let result = (|| {
            if matches!(self.source, ReplacementSource::New) != document.is_none() {
                return Err(ApplicationFailure::InvalidEffect);
            }
            let mut editor = match self.source {
                ReplacementSource::Recovery(_) => EditorSession::new_recovered(
                    self.capabilities,
                    self.tool,
                    self.brush,
                    document.unwrap(),
                ),
                _ => EditorSession::new(self.capabilities, self.tool, self.brush, document),
            }
            .map_err(ApplicationFailure::Editor)?;
            if let ReplacementSource::Document(destination) = self.source {
                editor
                    .bind_loaded_destination(destination)
                    .map_err(ApplicationFailure::Storage)?;
            }
            Ok(PreparedReplacement {
                token: self.token,
                revision: self.revision,
                editor: Some(editor),
            })
        })();
        result.map_err(|failure| ReplacementFailure {
            token: self.token,
            failure,
        })
    }
}
impl ApplicationSession {
    pub(super) fn install(
        &mut self,
        candidate: &mut PreparedReplacement,
    ) -> Result<EditorSession, ApplicationFailure> {
        let Some(Pending::Replacement(token, revision)) = self.pending else {
            return Err(ApplicationFailure::InvalidEffect);
        };
        if token != candidate.token || revision != candidate.revision || candidate.editor.is_none()
        {
            return Err(ApplicationFailure::InvalidEffect);
        }
        let state = self.editor.view_state();
        if state.session != token.session || state.revision != revision {
            return Err(ApplicationFailure::StaleDocument);
        }
        if state.interaction != InteractionState::Idle || self.explicit_pending() {
            return Err(ApplicationFailure::Busy);
        }
        let mut editor = candidate.editor.take().unwrap();
        editor.inherit_settings(state.tool, state.brush);
        self.pending = None;
        self.navigation = None;
        Ok(std::mem::replace(&mut self.editor, editor))
    }
    pub(super) fn prepare(
        &mut self,
        token: EffectToken,
        source: ReplacementSource,
    ) -> ApplicationEffect {
        let state = self.editor.view_state();
        self.pending = Some(Pending::Replacement(token, state.revision));
        ApplicationEffect::PrepareReplacement(ReplacementTicket {
            token,
            source,
            revision: state.revision,
            capabilities: state.capabilities,
            tool: state.tool,
            brush: state.brush,
        })
    }
}
