use crate::dto;
use sketch_core::application::{routing::*, *};
pub fn brush(value: dto::Brush) -> Result<BrushSettings, u32> {
    BrushSettings::try_from(sketch_core::Brush {
        rgba: [value.red, value.green, value.blue, value.alpha],
        width: value.width,
    })
    .map_err(|_| 1)
}
pub fn tool(value: u32) -> Result<ToolId, u32> {
    match value {
        1 => Ok(ToolId::Pen),
        2 => Ok(ToolId::Select),
        _ => Err(1),
    }
}
pub fn file(tag: u32) -> Option<FileIntent> {
    Some(match tag {
        20 => FileIntent::Save,
        21 => FileIntent::SaveAs,
        22 => FileIntent::ExportCopy,
        23 => FileIntent::Navigate(NavigationIntent::New),
        24 => FileIntent::Navigate(NavigationIntent::Open),
        25 => FileIntent::Navigate(NavigationIntent::Close),
        _ => return None,
    })
}
pub fn command(value: dto::Command) -> Result<ApplicationCommand<'static>, u32> {
    if value.reserved != 0 {
        return Err(1);
    }
    if let Some(intent) = file(value.tag) {
        return Ok(ApplicationCommand::File(intent));
    }
    // C union bytes have fixed scalar layouts; tag selects the only meaningful field.
    let edit = unsafe {
        match value.tag {
            1 => EditorCommand::SetTool(tool(value.payload.tool)?),
            2 => EditorCommand::SetBrush(brush(value.payload.brush)?),
            3 => {
                let v = value.payload.selection;
                EditorCommand::SelectAt {
                    x: v.x,
                    y: v.y,
                    tolerance: v.tolerance,
                }
            }
            4 => EditorCommand::SelectCycle(value.payload.direction),
            5 => EditorCommand::ClearSelection,
            6 => {
                let v = value.payload.stroke;
                if v.reserved != 0 {
                    return Err(1);
                }
                EditorCommand::BeginStroke {
                    pointer: PointerId(v.pointer),
                    device: match v.device {
                        0 => DeviceKind::Mouse,
                        1 => DeviceKind::Pen,
                        2 => DeviceKind::Touch,
                        _ => return Err(1),
                    },
                }
            }
            7 => {
                let v = value.payload.stroke;
                if v.reserved != 0 || v.device != 0 {
                    return Err(1);
                }
                EditorCommand::CommitStroke(PointerId(v.pointer))
            }
            8 => EditorCommand::BeginTransform,
            9 => {
                let v = value.payload.transform;
                EditorCommand::UpdateTransform {
                    scale: v.scale,
                    anchor_x: v.anchor_x,
                    anchor_y: v.anchor_y,
                }
            }
            10 => EditorCommand::CommitTransform,
            11 => EditorCommand::CancelInteraction,
            12 => EditorCommand::DeleteSelection,
            13 => EditorCommand::Undo,
            14 => EditorCommand::Redo,
            _ => return Err(1),
        }
    };
    Ok(ApplicationCommand::Edit(edit))
}
pub fn availability(tag: u32) -> Option<ActionId> {
    Some(match tag {
        101 => ActionId::ChoosePen,
        102 => ActionId::ChooseSelect,
        2 => ActionId::SetBrush,
        3..=5 => ActionId::Select,
        6 => ActionId::BeginStroke,
        7 => ActionId::CommitStroke,
        8 => ActionId::BeginTransform,
        9 => ActionId::UpdateTransform,
        10 => ActionId::CommitTransform,
        11 => ActionId::Cancel,
        12 => ActionId::Delete,
        13 => ActionId::Undo,
        14 => ActionId::Redo,
        _ => return None,
    })
}
