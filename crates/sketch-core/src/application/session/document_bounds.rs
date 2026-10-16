use super::*;
impl EditorSession {
    /// Query only: does not prepare scenes, clone the document, or include temporary work.
    pub(super) fn document_bounds(&self) -> Option<crate::Bounds> {
        self.editor
            .document()
            .strokes
            .iter()
            .fold(None, |bounds, stroke| {
                let next = crate::geometry::bounds(stroke, stroke.transform);
                Some(match bounds {
                    None => next,
                    Some(previous) => {
                        let x = previous.x.min(next.x);
                        let y = previous.y.min(next.y);
                        crate::Bounds {
                            x,
                            y,
                            width: (previous.x + previous.width).max(next.x + next.width) - x,
                            height: (previous.y + previous.height).max(next.y + next.height) - y,
                        }
                    }
                })
            })
    }
}
