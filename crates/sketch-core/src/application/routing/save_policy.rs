use super::*;
/// Immutable routing behavior; document/save acknowledgment semantics are unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SaveDestinationPolicy {
    /// Desktop document saving: reuse a confirmed binding when available.
    #[default]
    ReuseBinding,
    /// Every Save, including a navigation guard, requires a fresh destination choice.
    AlwaysChoose,
}
impl ApplicationSession {
    pub(super) fn reusable_save_destination(&self) -> Option<DestinationId> {
        match self.save_policy {
            SaveDestinationPolicy::ReuseBinding => self.editor.save_state().destination(),
            SaveDestinationPolicy::AlwaysChoose => None,
        }
    }
}
