use crate::*;
impl Reply {
    pub(crate) fn projection(&mut self, session: u64, p: ApplicationProjection, continuation: u64) {
        let e = p.editor;
        let d = e.document;
        let (interaction, kind) = match e.interaction {
            InteractionState::Idle => (0, 0),
            InteractionState::Drawing(id) => (id.get(), 1),
            InteractionState::Transforming(id) => (id.get(), 2),
        };
        for v in [
            session,
            e.session.get(),
            e.revision.0,
            e.scene_generation.get(),
            interaction,
            d.count as u64,
            d.selected,
            p.save.destination().map_or(0, |v| v.0),
            p.save.binding_generation(),
        ] {
            self.u64(v);
        }
        let tool = match e.tool {
            ToolId::Pen => 1,
            ToolId::Select => 2,
        };
        let caps = u32::from(e.capabilities.edit_document)
            | (u32::from(e.capabilities.export_document) << 1);
        let flags = u32::from(d.dirty)
            | (u32::from(d.can_undo) << 1)
            | (u32::from(d.can_redo) << 2)
            | (u32::from(p.save.binding_pending()) << 3);
        let navigation = match p.navigation {
            None => 0,
            Some(NavigationIntent::New) => 1,
            Some(NavigationIntent::Open) => 2,
            Some(NavigationIntent::Close) => 3,
            Some(NavigationIntent::Recover(_)) => 4,
        };
        for v in [
            tool,
            kind,
            caps,
            flags,
            write_tags::phase(p.file),
            navigation,
            p.outstanding_saves,
            0,
        ] {
            self.u32(v);
        }
        let brush = e.brush.brush();
        for v in brush.rgba {
            self.f64(v);
        }
        self.f64(brush.width);
        self.u64(continuation);
    }
    pub(crate) fn save(&mut self, save: Option<AppliedSave>) {
        let Some(save) = save else {
            self.zeros(72);
            return;
        };
        let (result, durability, failure) = write_tags::save_result(save.result);
        let t = save.token;
        for v in [
            1,
            tags::purpose(t.purpose()),
            result,
            write_tags::ack(save.acknowledgement),
            durability,
            failure,
        ] {
            self.u32(v);
        }
        for v in [
            t.session().get(),
            t.revision().0,
            t.destination().0,
            t.sequence(),
            t.binding_generation(),
            t.request().0,
        ] {
            self.u64(v);
        }
    }
}
impl WireApp {
    pub(crate) fn encode_outcome(
        &mut self,
        reply: &mut Reply,
        handle: SessionHandle,
        outcome: AppOutcome,
        fresh: u64,
    ) {
        let continuation = self.track(handle, outcome.projection, fresh);
        reply.projection(self.session_ref(handle), outcome.projection, continuation);
        let c = outcome.changes;
        reply.u32(
            u32::from(c.document)
                | (u32::from(c.session_replaced) << 1)
                | (u32::from(c.settings) << 2)
                | (u32::from(c.selection) << 3)
                | (u32::from(c.interaction) << 4),
        );
        reply.u32(write_tags::invalidation(outcome.invalidation));
        let mut argument = 0;
        let mut effect = 0;
        let mut operation = 0;
        let mut revision = 0;
        let mut failure = Failure::default();
        let tag = match outcome.effect {
            None => 0,
            Some(HostEffect::ChooseDestination { token: _, purpose }) => {
                effect = continuation;
                argument = tags::purpose(purpose);
                1
            }
            Some(HostEffect::ChooseSource(_)) => {
                effect = continuation;
                2
            }
            Some(HostEffect::ConfirmUnsaved {
                token: _,
                revision: r,
            }) => {
                effect = continuation;
                revision = r.0;
                3
            }
            Some(HostEffect::AwaitWrites(_)) => {
                effect = continuation;
                4
            }
            Some(HostEffect::WorkQueued(id)) => {
                operation = self.register_operation(
                    id,
                    handle,
                    fresh + 1,
                    outcome.projection.file != FilePhase::Replacement,
                    false,
                );
                5
            }
            Some(HostEffect::NavigationReady(_)) => {
                effect = continuation;
                6
            }
            Some(HostEffect::CloseConfirmed) => 7,
            Some(HostEffect::Cancelled) => 8,
            Some(HostEffect::Failure(value)) => {
                failure = value.into();
                9
            }
        };
        reply.u32(tag);
        reply.u32(argument);
        reply.u64(effect);
        reply.u64(operation);
        reply.u64(revision);
        reply.raw(&failure.bytes());
        reply.save(outcome.save);
    }
}
