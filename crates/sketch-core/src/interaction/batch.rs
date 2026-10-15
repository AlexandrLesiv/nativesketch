use crate::{Error, Input, MAX_SAMPLES, Result, Sample};
/// Validates against the pre-batch actual sequence before any transient mutation.
pub(super) fn validate(actual: &[Sample], inputs: &[Input]) -> Result<()> {
    if inputs.len() > MAX_SAMPLES
        || actual.len().saturating_add(
            inputs
                .iter()
                .filter(|x| matches!(x, Input::Actual(_)))
                .count(),
        ) > MAX_SAMPLES
    {
        return Err(Error::InvalidArgument);
    }
    // Validate the entire batch before changing even transient state. Actual ids
    // strictly increase, allowing correction lookup without copying ink history.
    let mut last = actual.last().copied();
    let mut prediction_started = false;
    for input in inputs {
        let s = match input {
            Input::Actual(s) | Input::Predicted(s) | Input::Correction(s) => *s,
        };
        s.validate()?;
        match input {
            Input::Actual(_) => {
                if prediction_started
                    || last.is_some_and(|p| s.id <= p.id || s.timestamp < p.timestamp)
                {
                    return Err(Error::InvalidArgument);
                }
                last = Some(s);
            }
            Input::Predicted(_) => {
                prediction_started = true;
                if last.is_some_and(|p| s.timestamp < p.timestamp) {
                    return Err(Error::InvalidArgument);
                }
            }
            Input::Correction(_) => {
                let i = actual
                    .binary_search_by_key(&s.id, |p| p.id)
                    .map_err(|_| Error::InvalidArgument)?;
                if s.timestamp != actual[i].timestamp {
                    return Err(Error::InvalidArgument);
                }
            }
        }
    }
    Ok(())
}
