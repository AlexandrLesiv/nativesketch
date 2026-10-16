/// Product capability policy, not authentication or an agent permission mechanism.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capabilities {
    pub edit_document: bool,
    /// Reserved for task 021 export effects; queries are available regardless of this bit.
    pub export_document: bool,
}
