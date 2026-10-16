use super::*;
#[test]
fn explicit_output_enum_mappings_match_named_golden_units() {
    for (name, value) in [
        ("Construct", Operation::Construct),
        ("SetTool", Operation::SetTool),
        ("SetBrush", Operation::SetBrush),
        ("Select", Operation::Select),
        ("BeginStroke", Operation::BeginStroke),
        ("Append", Operation::Append),
        ("CommitStroke", Operation::CommitStroke),
        ("BeginTransform", Operation::BeginTransform),
        ("UpdateTransform", Operation::UpdateTransform),
        ("CommitTransform", Operation::CommitTransform),
        ("Cancel", Operation::Cancel),
        ("Delete", Operation::Delete),
        ("Undo", Operation::Undo),
        ("Redo", Operation::Redo),
        ("New", Operation::New),
        ("Replace", Operation::Replace),
        ("Query", Operation::Query),
    ] {
        assert_eq!(
            tags::operation(value).to_le_bytes().as_slice(),
            fixture(&format!("tag.operation.{name}"))
        );
    }
    for (name, value) in [
        ("Save", SavePurpose::Save),
        ("SaveAs", SavePurpose::SaveAs),
        ("Checkpoint", SavePurpose::Checkpoint),
        ("ExportCopy", SavePurpose::ExportCopy),
    ] {
        assert_eq!(
            tags::purpose(value).to_le_bytes().as_slice(),
            fixture(&format!("tag.purpose.{name}"))
        );
    }
    for (name, value) in [
        ("SavedCurrent", Acknowledgement::SavedCurrent),
        ("SavedOlder", Acknowledgement::SavedOlder),
        ("CopyWritten", Acknowledgement::CopyWritten),
        ("CheckpointWritten", Acknowledgement::CheckpointWritten),
        ("StaleSession", Acknowledgement::StaleSession),
        ("StaleBinding", Acknowledgement::StaleBinding),
        ("Superseded", Acknowledgement::Superseded),
        ("Cancelled", Acknowledgement::Cancelled),
        ("Failed", Acknowledgement::Failed),
        ("CopyDispatched", Acknowledgement::CopyDispatched),
    ] {
        assert_eq!(
            write_tags::ack(value).to_le_bytes().as_slice(),
            fixture(&format!("tag.ack.{name}"))
        );
    }
    for (name, value) in [
        ("FileAndDirectorySynced", Durability::FileAndDirectorySynced),
        (
            "DirectorySyncUnconfirmed",
            Durability::DirectorySyncUnconfirmed,
        ),
        ("ProviderConfirmed", Durability::ProviderConfirmed),
    ] {
        assert_eq!(
            write_tags::durability(value).to_le_bytes().as_slice(),
            fixture(&format!("tag.durability.{name}"))
        );
    }
    for (name, value) in [
        ("Idle", FilePhase::Idle),
        ("Destination", FilePhase::Destination),
        ("Source", FilePhase::Source),
        ("Unsaved", FilePhase::Unsaved),
        ("Writes", FilePhase::Writes),
        ("Replacement", FilePhase::Replacement),
        ("CloseReady", FilePhase::CloseReady),
    ] {
        assert_eq!(
            write_tags::phase(value).to_le_bytes().as_slice(),
            fixture(&format!("tag.phase.{name}"))
        );
    }
    for (name, value) in [
        ("None", RenderInvalidation::None),
        ("Overlay", RenderInvalidation::Overlay),
        ("Full", RenderInvalidation::Full),
    ] {
        assert_eq!(
            write_tags::invalidation(value).to_le_bytes().as_slice(),
            fixture(&format!("tag.invalidation.{name}"))
        );
    }
    for (name, value) in [
        ("ReadOnly", DisabledReason::ReadOnly),
        ("InteractionActive", DisabledReason::InteractionActive),
        ("WrongTool", DisabledReason::WrongTool),
        ("NothingSelected", DisabledReason::NothingSelected),
        ("HistoryEmpty", DisabledReason::HistoryEmpty),
        ("NoActiveStroke", DisabledReason::NoActiveStroke),
        ("NoActiveTransform", DisabledReason::NoActiveTransform),
    ] {
        assert_eq!(
            write_tags::disabled(value).to_le_bytes().as_slice(),
            fixture(&format!("tag.availability.{name}"))
        );
    }
    for (name, value) in [
        ("Write", WorkKind::Write),
        ("Read", WorkKind::Read),
        ("New", WorkKind::PrepareNew),
        ("Retire", WorkKind::Retire),
    ] {
        assert_eq!(
            write_tags::work(value).to_le_bytes().as_slice(),
            fixture(&format!("tag.work.{name}"))
        );
    }
}
