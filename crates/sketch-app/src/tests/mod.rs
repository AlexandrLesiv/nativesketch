use crate::*;
use sketch_core::{Brush, Input, Sample};
mod aliases;
mod budgets;
mod diagnostics;
mod identity;
mod lifecycle;
mod loads;
mod navigation;
mod saves;
mod viewer;
const FIXTURE: &[u8] = include_bytes!("../../../sketch-core/tests/fixtures/portable-v1.nsketch");
fn app(limits: AppLimits, editable: bool) -> (PortableApp, SessionHandle) {
    let mut app = PortableApp::new(limits).unwrap();
    let handle = app.create_session(config(editable)).unwrap();
    (app, handle)
}
fn config(editable: bool) -> SessionConfig {
    SessionConfig {
        capabilities: Capabilities {
            edit_document: editable,
            export_document: editable,
        },
        tool: if editable {
            ToolId::Pen
        } else {
            ToolId::Select
        },
        brush: BrushSettings::try_from(Brush {
            rgba: [0.0, 0.0, 0.0, 1.0],
            width: 4.0,
        })
        .unwrap(),
    }
}
fn ctx(app: &PortableApp, handle: SessionHandle) -> CommandContext {
    let state = app.projection(handle).unwrap().editor;
    CommandContext {
        session: state.session,
        request: RequestId(42),
        interaction: match state.interaction {
            InteractionState::Idle => None,
            InteractionState::Drawing(id) | InteractionState::Transforming(id) => Some(id),
        },
    }
}
fn run(app: &mut PortableApp, handle: SessionHandle, intent: Intent<'_>) -> AppOutcome {
    app.dispatch(handle, ctx(app, handle), intent).unwrap()
}
fn draft(app: &mut PortableApp, handle: SessionHandle) {
    run(
        app,
        handle,
        Intent::Edit(EditorCommand::BeginStroke {
            pointer: PointerId(1),
            device: DeviceKind::Pen,
        }),
    );
    let sample = Sample {
        id: 1,
        x: 10.0,
        y: 20.0,
        timestamp: 1.0,
        pressure: Some(0.5),
    };
    run(
        app,
        handle,
        Intent::Input(SampleBatch {
            pointer: PointerId(1),
            samples: &[Input::Actual(sample)],
        }),
    );
}
fn stroke(app: &mut PortableApp, handle: SessionHandle) {
    draft(app, handle);
    run(
        app,
        handle,
        Intent::Edit(EditorCommand::CommitStroke(PointerId(1))),
    );
}
fn resource(id: u64) -> ResourceDescriptor {
    ResourceDescriptor {
        provider: ProviderId(1),
        resource: ProviderResourceId(id),
        canonical: ProviderResourceId(id),
        access: ResourceAccess {
            read: true,
            write: true,
            confirmed_write: true,
        },
        generation: 1,
    }
}
fn save(
    app: &mut PortableApp,
    handle: SessionHandle,
    descriptor: ResourceDescriptor,
    export: bool,
) -> OperationId {
    let outcome = run(
        app,
        handle,
        Intent::File(if export {
            FileIntent::ExportCopy
        } else {
            FileIntent::SaveAs
        }),
    );
    let Some(HostEffect::ChooseDestination { token, .. }) = outcome.effect else {
        panic!()
    };
    let outcome = app
        .supply_resource(handle, ctx(app, handle), token, descriptor)
        .unwrap();
    let Some(HostEffect::WorkQueued(id)) = outcome.effect else {
        panic!()
    };
    id
}
fn write(app: &mut PortableApp) -> EncodedWrite {
    let Some(WorkItem::Write(work)) = app.take_work().unwrap() else {
        panic!()
    };
    work.encode().unwrap()
}
fn complete(app: &mut PortableApp, result: WorkResult) {
    let mut result = Some(result);
    app.complete_work(&mut result).unwrap();
    assert!(result.is_none());
}
fn written(work: EncodedWrite) -> WorkResult {
    work.finish(WriteOutcome::Written(Durability::ProviderConfirmed))
        .unwrap()
}
fn open(
    app: &mut PortableApp,
    handle: SessionHandle,
    descriptor: ResourceDescriptor,
) -> OperationId {
    let outcome = run(
        app,
        handle,
        Intent::File(FileIntent::Navigate(NavigationIntent::Open)),
    );
    let Some(HostEffect::ChooseSource(token)) = outcome.effect else {
        panic!("clean source chooser")
    };
    let outcome = app
        .supply_resource(handle, ctx(app, handle), token, descriptor)
        .unwrap();
    let Some(HostEffect::WorkQueued(id)) = outcome.effect else {
        panic!()
    };
    id
}
fn read(app: &mut PortableApp) -> ProviderRead {
    let Some(WorkItem::Read(work)) = app.take_work().unwrap() else {
        panic!()
    };
    work
}
fn retire(app: &mut PortableApp) {
    while let Some(work) = app.take_work().unwrap() {
        let WorkItem::Retire(work) = work else {
            panic!("unexpected nonretirement work")
        };
        complete(app, work.dispose());
    }
}
