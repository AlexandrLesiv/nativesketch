using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;

namespace NativeSketch;

internal sealed class EditorPresenter
{
    readonly Window window;
    readonly EditorView view;
    readonly CanvasViewport viewport;
    readonly DocumentEffects documents;
    internal EditorPresenter(Window window, EditorView view, CanvasViewport viewport, DocumentEffects documents)
    {
        this.window = window;
        this.view = view;
        this.viewport = viewport;
        this.documents = documents;
    }

    internal void Refresh(State s)
    {
        view.status.Text = $"{(s.Dirty != 0 ? "Unsaved · " : "")}{s.StrokeCount} strokes · selected {s.SelectedId} · {viewport.zoom:P0}" + (s.StrokeCount == 0 ? " · Draw with Pen; Space-drag to pan" : "");
        AutomationProperties.SetName(view.canvas, "Drawing canvas. " + view.status.Text);
        foreach (var item in view.menuItems.Values)
            item.IsEnabled = !documents.busy;
        view.undo.IsEnabled = view.menuItems["Undo"].IsEnabled = !documents.busy && s.CanUndo != 0;
        view.redo.IsEnabled = view.menuItems["Redo"].IsEnabled = !documents.busy && s.CanRedo != 0;
        view.apply.IsEnabled = view.menuItems["Delete selection"].IsEnabled = !documents.busy && s.SelectedId != 0;
        view.menuItems["Next stroke"].IsEnabled = view.menuItems["Previous stroke"].IsEnabled = !documents.busy && s.StrokeCount != 0;
        window.Title = (documents.file == null ? "Untitled" : System.IO.Path.GetFileName(documents.file)) + (s.Dirty != 0 ? " *" : "") + " — Native Sketch";
    }
}
