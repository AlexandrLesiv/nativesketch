use crate::*;
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Failure {
    pub domain: u32,
    pub code: u32,
    pub operation: u32,
    pub document: u64,
    pub request: u64,
    pub interaction: u64,
}
impl Failure {
    pub(crate) fn new(domain: u32, code: u32) -> Self {
        Self {
            domain,
            code,
            ..Self::default()
        }
    }
    pub(crate) fn context(mut self, context: CommandContext) -> Self {
        self.document = context.session.get();
        self.request = context.request.0;
        self.interaction = context.interaction.map_or(0, InteractionId::get);
        self
    }
    pub fn bytes(self) -> [u8; 40] {
        let mut bytes = [0; 40];
        for (i, value) in [self.domain, self.code, self.operation, 0]
            .into_iter()
            .enumerate()
        {
            bytes[i * 4..i * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        for (i, value) in [self.document, self.request, self.interaction]
            .into_iter()
            .enumerate()
        {
            bytes[16 + i * 8..24 + i * 8].copy_from_slice(&value.to_le_bytes());
        }
        bytes
    }
}
impl From<EditorError> for Failure {
    fn from(v: EditorError) -> Self {
        let mut f = Self::new(1, crate::tags::failure_kind(v.kind));
        f.operation = crate::tags::operation(v.operation);
        if let Some(ctx) = v.context {
            f = f.context(ctx);
        }
        f
    }
}
impl From<ApplicationFailure> for Failure {
    fn from(v: ApplicationFailure) -> Self {
        match v {
            ApplicationFailure::Editor(v) => v.into(),
            ApplicationFailure::Storage(v) => Self::new(2, crate::tags::storage_error(v)),
            ApplicationFailure::Write(v) => Self::new(3, crate::tags::write_failure(v)),
            ApplicationFailure::InvalidEffect => Self::new(4, 1),
            ApplicationFailure::StaleDocument => Self::new(4, 2),
            ApplicationFailure::Busy => Self::new(4, 3),
            ApplicationFailure::CapabilityDenied => Self::new(4, 4),
            ApplicationFailure::NavigationRequired => Self::new(4, 5),
            ApplicationFailure::ResourceLimit => Self::new(4, 6),
        }
    }
}
