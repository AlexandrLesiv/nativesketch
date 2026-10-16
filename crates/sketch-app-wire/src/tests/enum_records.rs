use super::*;
#[test]
fn native_tool_device_direction_and_projection_tags_are_explicit() {
    for (name, tool) in [("Pen", 1u32), ("Select", 2)] {
        let mut bytes = fixture("action.1");
        bytes[56..60].copy_from_slice(&fixture(&format!("tag.tool.{name}")));
        let Request::Action(_, EditorAction::ChooseTool(value)) = decode(&bytes).unwrap().command
        else {
            panic!()
        };
        assert_eq!(
            value,
            if tool == 1 {
                ToolId::Pen
            } else {
                ToolId::Select
            }
        );
    }
    for (name, value) in [
        ("Mouse", DeviceKind::Mouse),
        ("Pen", DeviceKind::Pen),
        ("Touch", DeviceKind::Touch),
    ] {
        let mut bytes = fixture("edit.6");
        bytes[64..68].copy_from_slice(&fixture(&format!("tag.device.{name}")));
        let Request::Edit(_, EditorCommand::BeginStroke { device, .. }) =
            decode(&bytes).unwrap().command
        else {
            panic!()
        };
        assert_eq!(device, value);
    }
    for (name, value) in [
        ("Previous", CycleDirection::Previous),
        ("Next", CycleDirection::Next),
    ] {
        let mut bytes = fixture("action.2");
        bytes[56..60].copy_from_slice(&fixture(&format!("tag.direction.{name}")));
        let Request::Action(_, EditorAction::CycleSelection(direction)) =
            decode(&bytes).unwrap().command
        else {
            panic!()
        };
        assert_eq!(direction, value);
    }
    let mut a = app();
    let id = session(&mut a);
    let mut begin = words(&[7]);
    begin.extend([2, 0, 0, 0, 0, 0, 0, 0]);
    drop(edit(&mut a, id, 6, &begin));
    let mut p = a.app.projection(a.session(id).unwrap()).unwrap();
    let InteractionState::Drawing(interaction) = p.editor.interaction else {
        panic!()
    };
    for (name, state) in [
        ("Idle", InteractionState::Idle),
        ("Drawing", InteractionState::Drawing(interaction)),
        ("Transforming", InteractionState::Transforming(interaction)),
    ] {
        p.editor.interaction = state;
        let mut reply = Reply::reserve(&a.budget, 3, 1, 4096).unwrap();
        reply.projection(id, p, 0);
        assert_eq!(
            &reply.finish().bytes()[148..152],
            fixture(&format!("tag.interaction.{name}"))
        );
    }
    for (name, navigation) in [
        ("None", None),
        ("New", Some(NavigationIntent::New)),
        ("Open", Some(NavigationIntent::Open)),
        ("Close", Some(NavigationIntent::Close)),
        (
            "Recover",
            Some(NavigationIntent::Recover(RecoverySourceId(1))),
        ),
    ] {
        p.navigation = navigation;
        let mut reply = Reply::reserve(&a.budget, 3, 1, 4096).unwrap();
        reply.projection(id, p, 0);
        assert_eq!(
            &reply.finish().bytes()[164..168],
            fixture(&format!("tag.navigation.{name}"))
        );
    }
}
#[test]
fn stale_or_foreign_effect_and_operation_references_cannot_execute() {
    let mut a = app();
    let id = session(&mut a);
    let reply = file(&mut a, id, 2);
    let effect = u64_at(reply.bytes(), 216);
    drop(reply);
    let mut b = app();
    let other = session(&mut b);
    let reply = file(&mut b, other, 2);
    let second = u64_at(reply.bytes(), 216);
    drop(reply);
    assert_ne!(effect, second);
    let mut bytes = context(&mut b, other);
    bytes.extend(words(&[effect]));
    bytes.extend([1, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(b.call(&packet(21, &bytes)).unwrap_err().status, 7);
    let reply = supply(&mut a, id, effect, 1, 1, 7);
    let operation = u64_at(reply.bytes(), 248);
    drop(reply);
    assert_eq!(
        b.call(&packet(33, &words(&[operation])))
            .unwrap_err()
            .status,
        7
    );
    let mut bytes = context(&mut a, id);
    bytes.extend(words(&[effect]));
    bytes.extend([1, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(a.call(&packet(21, &bytes)).unwrap_err().status, 7);
}
