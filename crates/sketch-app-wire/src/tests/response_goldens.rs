use super::*;
const H: u64 = 0xf123456789abcd00;
const REQUEST: u64 = 0xfedcba9876543210;
fn compare_doc(mut bytes: Vec<u8>, offset: usize, actual: u64, name: &str) {
    assert_eq!(u64_at(&bytes, offset), actual);
    bytes[offset..offset + 8].copy_from_slice(&(H + 32).to_le_bytes());
    assert_eq!(bytes, fixture(name));
}
#[test]
fn complete_projection_and_outcome_layouts_match_fixed_bytes() {
    let mut a = app();
    let id = session(&mut a);
    let handle = a.session(id).unwrap();
    let raw = RawContext {
        session: id,
        document: a.app.projection(handle).unwrap().editor.session.get(),
        interaction: 0,
    };
    let (_, ctx) = a.context(raw, 17).unwrap();
    let value = a
        .app
        .dispatch(handle, ctx, Intent::File(FileIntent::SaveAs))
        .unwrap();
    let token = value.projection.continuation.unwrap();
    let mut p = value.projection;
    p.editor.revision = Revision(23);
    p.editor.document.count = 3;
    p.editor.document.selected = 17;
    p.editor.document.dirty = true;
    p.editor.document.can_undo = true;
    p.file = FilePhase::Unsaved;
    p.navigation = Some(NavigationIntent::Open);
    p.outstanding_saves = 2;
    let mut reply = Reply::reserve(&a.budget, 3, REQUEST, 4096).unwrap();
    reply.projection(H + 16, p, H + 48);
    compare_doc(
        reply.finish().bytes().to_vec(),
        80,
        p.editor.session.get(),
        "response.projection",
    );
    a.sessions[0].id = H + 16;
    let outcome = AppOutcome {
        projection: p,
        changes: StateChanges {
            document: true,
            session_replaced: false,
            settings: true,
            selection: true,
            interaction: false,
        },
        invalidation: RenderInvalidation::Full,
        effect: Some(HostEffect::ConfirmUnsaved {
            token,
            revision: Revision(23),
        }),
        save: None,
    };
    let mut reply = Reply::reserve(&a.budget, 20, REQUEST, 4096).unwrap();
    a.encode_outcome(&mut reply, handle, outcome, H + 48);
    compare_doc(
        reply.finish().bytes().to_vec(),
        80,
        p.editor.session.get(),
        "response.outcome",
    );
}
#[test]
fn complete_work_info_layout_has_high_bit_opaque_ids() {
    let mut a = app();
    let id = session(&mut a);
    draw(&mut a, id);
    let _ = queued(&mut a, id, 2, H + 64, 7);
    let mut work = take(&mut a);
    work.operation = H + 16;
    work.session = H + 32;
    let mut reply = Reply::reserve(&a.budget, 100, REQUEST, 4096).unwrap();
    work.info_reply(&mut reply).unwrap();
    assert_eq!(reply.finish().bytes(), fixture("response.work"));
}
#[test]
fn high_bit_payload_values_decode_without_float_conversion() {
    let Request::Resource(c, effect, r) = decode(&fixture("high.resource")).unwrap().command else {
        panic!()
    };
    assert_eq!(c.session, H + 16);
    assert_eq!(c.document, H + 32);
    assert_eq!(c.interaction, H + 48);
    assert_eq!(effect, H + 64);
    assert_eq!(r.resource.0, H + 64);
    assert_eq!(r.canonical.0, H + 80);
    let Request::Input(_, pointer, samples) = decode(&fixture("high.sample")).unwrap().command
    else {
        panic!()
    };
    assert_eq!(pointer.0, H + 64);
    let sketch_core::Input::Actual(s) = samples[0] else {
        panic!()
    };
    assert_eq!(s.id, H + 80);
}
