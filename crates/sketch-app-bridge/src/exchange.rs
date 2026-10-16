use crate::{objects::*, owner::with_app, registry::*, *};
#[derive(Debug)]
pub struct WorkOutput {
    pub reply: Handle,
    pub result: Handle,
    pub work_consumed: bool,
    pub input_consumed: bool,
}
pub fn app_take_work(id: Handle, packet: &[u8]) -> Result<(Handle, Handle)> {
    control(packet)?;
    with_app(id, |app, meta| {
        let reply = Slot::reserve(Kind::Buffer, Some(meta.clone()))?;
        let work = Slot::reserve(Kind::Work, Some(meta))?;
        let (r, w) = app.take_work(packet)?;
        let w = w.map_or(0, |w| work.commit(Object::Work(w), None));
        Ok((reply.commit(Object::Buffer(r), None), w))
    })
}
pub fn work_copy_input(id: Handle, bytes: &[u8]) -> Result<Handle> {
    let held = Checkout::take(id, Kind::Work)?;
    let meta = held.owner();
    meta.worker()?;
    let slot = Slot::reserve(Kind::Input, Some(meta.clone()))?;
    guarded(&meta, || {
        let Some(Object::Work(work)) = &held.object else {
            unreachable!()
        };
        let input = work.copy_input(bytes)?;
        Ok(slot.commit(Object::Input(Some(input)), None))
    })
}
pub fn work_call(id: Handle, packet: &[u8], input: Handle) -> Result<WorkOutput> {
    control(packet)?;
    let mut held = Checkout::take(id, Kind::Work)?;
    let meta = held.owner();
    meta.worker()?;
    let mut bytes = if input == 0 {
        None
    } else {
        Some(Checkout::take(input, Kind::Input)?)
    };
    if bytes.as_ref().is_some_and(|b| b.owner().id != meta.id) {
        return Err(error(11));
    }
    let reply = Slot::reserve(Kind::Buffer, Some(meta.clone()))?;
    let result = Slot::reserve(Kind::Result, Some(meta.clone()))?;
    guarded(&meta, || {
        let Some(Object::Work(work)) = &mut held.object else {
            unreachable!()
        };
        let mut empty = None;
        let source = match &mut bytes {
            Some(b) => match &mut b.object {
                Some(Object::Input(v)) => v,
                _ => unreachable!(),
            },
            None => &mut empty,
        };
        let output = work.call(packet, source)?;
        let input_consumed = input != 0 && source.is_none();
        let work_consumed = output.result.is_some();
        let result = output
            .result
            .map_or(0, |r| result.commit(Object::Result(Some(r)), None));
        if work_consumed {
            held.consume();
        }
        if input_consumed {
            bytes.as_mut().expect("supplied input").consume();
        }
        Ok(WorkOutput {
            reply: reply.commit(Object::Buffer(output.reply), None),
            result,
            work_consumed,
            input_consumed,
        })
    })
}
pub fn app_complete(id: Handle, packet: &[u8], result: Handle) -> Result<Handle> {
    control(packet)?;
    with_app(id, |app, meta| {
        let mut held = Checkout::take(result, Kind::Result)?;
        if held.owner().id != meta.id {
            return Err(error(11));
        }
        let slot = Slot::reserve(Kind::Buffer, Some(meta))?;
        let Some(Object::Result(value)) = &mut held.object else {
            unreachable!()
        };
        let reply = app.complete(packet, value)?;
        held.consume();
        Ok(slot.commit(Object::Buffer(reply), None))
    })
}
