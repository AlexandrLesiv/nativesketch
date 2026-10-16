use crate::{sample, state};
use sketch_core::{Brush, application::*};

/// At most 128 commands per input. Rejected commands must preserve committed bytes,
/// projections and live geometry; every accepted committed snapshot must remain decodable.
pub fn check(data: &[u8]) {
    let mut session = EditorSession::new(
        Capabilities {
            edit_document: true,
            export_document: true,
        },
        ToolId::Pen,
        BrushSettings::try_from(Brush {
            rgba: [0.2, 0.3, 0.4, 0.5],
            width: 6.0,
        })
        .unwrap(),
        None,
    )
    .unwrap();
    let stale = state::context(&session, 1);
    for (step, bytes) in data.as_chunks::<8>().0.iter().take(128).enumerate() {
        let step = step as u64;
        let before = session.view_state();
        let document = state::snapshot(&session);
        let geometry = state::scene(&session);
        let context = if bytes[1] & 1 == 0 {
            state::context(&session, step + 2)
        } else {
            stale
        };
        let pointer = PointerId(if bytes[1] & 2 == 0 { 1 } else { 2 });
        let result = if bytes[0] % 16 == 1 {
            session.append(
                context,
                SampleBatch {
                    pointer,
                    samples: &sample::batch(bytes, step),
                },
            )
        } else if bytes[0] % 16 >= 14 {
            session.action(
                context,
                if bytes[0] % 16 == 14 {
                    EditorAction::Undo
                } else {
                    EditorAction::ChooseTool(ToolId::Select)
                },
            )
        } else {
            session.execute(context, command(bytes, pointer))
        };
        if result.is_err() {
            assert_eq!(session.view_state(), before);
            assert_eq!(state::snapshot(&session), document);
            assert_eq!(state::scene(&session), geometry);
        }
        let snapshot = state::snapshot(&session);
        ValidatedDocument::decode(&snapshot).unwrap();
        state::scene(&session);
    }
}

fn command(bytes: &[u8], pointer: PointerId) -> EditorCommand {
    match bytes[0] % 16 {
        0 => EditorCommand::BeginStroke {
            pointer,
            device: DeviceKind::Pen,
        },
        2 => EditorCommand::CommitStroke(pointer),
        3 => EditorCommand::CancelInteraction,
        4 => EditorCommand::SetTool(if bytes[2] & 1 == 0 {
            ToolId::Pen
        } else {
            ToolId::Select
        }),
        5 => EditorCommand::SelectCycle(if bytes[2] & 1 == 0 { 1 } else { -1 }),
        6 => EditorCommand::DeleteSelection,
        7 => EditorCommand::Undo,
        8 => EditorCommand::Redo,
        9 => EditorCommand::BeginTransform,
        10 => EditorCommand::UpdateTransform {
            scale: sample::number(bytes[2]) / 10.0,
            anchor_x: sample::number(bytes[3]),
            anchor_y: sample::number(bytes[4]),
        },
        11 => EditorCommand::CommitTransform,
        12 => EditorCommand::NewDocument,
        13 => EditorCommand::SelectAt {
            x: sample::number(bytes[2]),
            y: sample::number(bytes[3]),
            tolerance: sample::number(bytes[4]),
        },
        _ => unreachable!(),
    }
}
