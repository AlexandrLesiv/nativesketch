use crate::dto;
use sketch_core::application::{CycleDirection, EditorAction, ScaleAnchor, ScaleFactor};
pub fn action(value: dto::EditorAction) -> Result<EditorAction, u32> {
    if value.tag != 6 && value.factor != 0.0 {
        return Err(1);
    }
    if matches!(value.tag, 3..=5) && value.option != 0 {
        return Err(1);
    }
    Ok(match value.tag {
        1 => EditorAction::ChooseTool(super::commands::tool(value.option)?),
        2 => EditorAction::CycleSelection(match value.option {
            1 => CycleDirection::Previous,
            2 => CycleDirection::Next,
            _ => return Err(1),
        }),
        3 => EditorAction::DeleteSelection,
        4 => EditorAction::Undo,
        5 => EditorAction::Redo,
        6 if value.option == 1 => EditorAction::ScaleSelection {
            factor: ScaleFactor::try_from(value.factor).map_err(|_| 1u32)?,
            anchor: ScaleAnchor::SelectionCenter,
        },
        _ => return Err(1),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_action_fields_never_become_typed_intents() {
        for (tag, option, factor) in [
            (0, 0, 0.0),
            (7, 0, 0.0),
            (1, 0, 0.0),
            (1, 3, 0.0),
            (2, 0, 0.0),
            (2, 3, 0.0),
            (3, 1, 0.0),
            (4, 1, 0.0),
            (5, 1, 0.0),
            (6, 0, 1.0),
            (6, 2, 1.0),
            (6, 1, 0.0),
            (6, 1, 100.1),
            (6, 1, f64::NAN),
            (6, 1, f64::INFINITY),
            (1, 1, f64::NAN),
            (2, 1, 1.0),
            (3, 0, 1.0),
        ] {
            assert!(
                action(dto::EditorAction {
                    tag,
                    option,
                    factor
                })
                .is_err()
            );
        }
        for (tag, option, factor) in [
            (1, 1, 0.0),
            (1, 2, 0.0),
            (2, 1, 0.0),
            (2, 2, 0.0),
            (3, 0, 0.0),
            (4, 0, 0.0),
            (5, 0, 0.0),
            (6, 1, 100.0),
        ] {
            assert!(
                action(dto::EditorAction {
                    tag,
                    option,
                    factor
                })
                .is_ok()
            );
        }
    }
}
