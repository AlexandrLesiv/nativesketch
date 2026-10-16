use crate::bridge;
#[derive(Default, Debug, PartialEq, Eq)]
pub(crate) struct Receipt {
    pub flags: u32,
    pub app: u64,
    pub reply: u64,
    pub work: u64,
    pub input: u64,
    pub result: u64,
    pub count: u64,
}
impl Receipt {
    /// Fixed-size stack publication: no allocation or JNI after bridge transfer.
    pub fn bytes(result: bridge::Result<Self>, correlation: (u32, u64)) -> (i32, [u8; 128]) {
        let mut bytes = [0; 128];
        match result {
            Ok(r) => {
                bytes[4..8].copy_from_slice(&r.flags.to_le_bytes());
                for (i, field) in [r.app, r.reply, r.work, r.input, r.result, r.count]
                    .into_iter()
                    .enumerate()
                {
                    bytes[8 + i * 8..16 + i * 8].copy_from_slice(&field.to_le_bytes());
                }
                (0, bytes)
            }
            Err(e) => {
                bytes[..4].copy_from_slice(&e.status.to_le_bytes());
                bytes[56..].copy_from_slice(&e.packet(correlation.0, correlation.1));
                (e.status as i32, bytes)
            }
        }
    }
}
pub(crate) fn error(status: u32) -> bridge::WireError {
    bridge::WireError {
        status,
        failure: bridge::Failure::default(),
    }
}
