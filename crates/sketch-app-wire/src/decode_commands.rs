use crate::{reader::Reader, *};
impl Reader<'_> {
    pub fn edit(&mut self) -> Result<EditorCommand, WireError> {
        let tag = self.u32()?;
        self.zero()?;
        Ok(match tag {
            1 => {
                let tool = self.tool()?;
                self.zero()?;
                EditorCommand::SetTool(tool)
            }
            2 => EditorCommand::SetBrush(self.brush()?),
            3 => EditorCommand::SelectAt {
                x: self.f64()?,
                y: self.f64()?,
                tolerance: self.f64()?,
            },
            4 => {
                let direction = i32::from_le_bytes(self.take()?);
                self.zero()?;
                EditorCommand::SelectCycle(direction)
            }
            5 => EditorCommand::ClearSelection,
            6 => {
                let pointer = PointerId(self.u64()?);
                let device = match self.u32()? {
                    1 => DeviceKind::Mouse,
                    2 => DeviceKind::Pen,
                    3 => DeviceKind::Touch,
                    _ => return Err(WireError::malformed()),
                };
                self.zero()?;
                EditorCommand::BeginStroke { pointer, device }
            }
            7 => EditorCommand::CommitStroke(PointerId(self.u64()?)),
            8 => EditorCommand::BeginTransform,
            9 => EditorCommand::UpdateTransform {
                scale: self.f64()?,
                anchor_x: self.f64()?,
                anchor_y: self.f64()?,
            },
            10 => EditorCommand::CommitTransform,
            11 => EditorCommand::CancelInteraction,
            12 => EditorCommand::DeleteSelection,
            13 => EditorCommand::Undo,
            14 => EditorCommand::Redo,
            _ => return Err(WireError::malformed()),
        })
    }
    pub fn action(&mut self) -> Result<EditorAction, WireError> {
        let tag = self.u32()?;
        self.zero()?;
        Ok(match tag {
            1 => {
                let tool = self.tool()?;
                self.zero()?;
                EditorAction::ChooseTool(tool)
            }
            2 => {
                let direction = match self.u32()? {
                    1 => CycleDirection::Previous,
                    2 => CycleDirection::Next,
                    _ => return Err(WireError::malformed()),
                };
                self.zero()?;
                EditorAction::CycleSelection(direction)
            }
            3 => EditorAction::DeleteSelection,
            4 => EditorAction::Undo,
            5 => EditorAction::Redo,
            6 => {
                if self.u32()? != 1 {
                    return Err(WireError::malformed());
                }
                self.zero()?;
                EditorAction::ScaleSelection {
                    anchor: ScaleAnchor::SelectionCenter,
                    factor: self.f64()?.try_into().map_err(|_| WireError::malformed())?,
                }
            }
            _ => return Err(WireError::malformed()),
        })
    }
    pub fn finish_write(&mut self) -> Result<WriteOutcome, WireError> {
        let tag = self.u32()?;
        let detail = self.u32()?;
        Ok(match tag {
            1 => WriteOutcome::Written(match detail {
                1 => Durability::FileAndDirectorySynced,
                2 => Durability::DirectorySyncUnconfirmed,
                3 => Durability::ProviderConfirmed,
                _ => return Err(WireError::malformed()),
            }),
            2 => WriteOutcome::Failed(Self::storage_failure(detail)?),
            3 => WriteOutcome::Uncertain(Self::storage_failure(detail)?),
            4 if detail == 0 => WriteOutcome::DispatchedUnconfirmed,
            _ => return Err(WireError::malformed()),
        })
    }
    pub fn storage_failure(tag: u32) -> Result<StorageFailure, WireError> {
        match tag {
            1 => Ok(StorageFailure::Encode),
            2 => Ok(StorageFailure::Access),
            3 => Ok(StorageFailure::Capacity),
            4 => Ok(StorageFailure::Write),
            5 => Ok(StorageFailure::Replace),
            6 => Ok(StorageFailure::WorkerStopped),
            _ => Err(WireError::malformed()),
        }
    }
}
