use crate::geometry::render_stroke;
use crate::interaction::ActiveInteraction;
use crate::{Document, PathCommand};
pub(crate) fn build(document: &Document, interaction: &ActiveInteraction) -> Vec<PathCommand> {
    let mut out = Vec::new();
    for (i, stroke) in document.strokes.iter().enumerate() {
        let transform = interaction.transform(i).unwrap_or(stroke.transform);
        render_stroke(stroke, transform, stroke.samples.iter().copied(), &mut out);
    }
    if let ActiveInteraction::Drawing(ink) = interaction {
        render_stroke(
            &ink.stroke,
            ink.stroke.transform,
            ink.stroke.samples.iter().chain(&ink.predicted).copied(),
            &mut out,
        );
    }
    out
}
