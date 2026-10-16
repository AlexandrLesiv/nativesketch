using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Automation;

namespace NativeSketch;

public sealed class SketchWindow : Window
{
    readonly Engine engine = new();
    readonly EditorView view = new();
    readonly CanvasViewport viewport = new();
    readonly CanvasInput input;
    readonly CanvasNavigation navigation;
    readonly CanvasPainter painter;
    readonly EditorPresenter presenter;
    readonly DocumentEffects documents;
    readonly NativeMenus menus;
    public SketchWindow()
    {
        documents = new DocumentEffects(this, engine, view, CancelInput, Draw);
        input = new CanvasInput(engine, view, viewport, Run, Draw, () => documents.busy);
        navigation = new CanvasNavigation(engine, view, viewport, input, Draw, Command);
        painter = new CanvasPainter(engine, view, viewport);
        presenter = new EditorPresenter(this, view, viewport, documents);
        menus = new NativeMenus(view, () => documents.busy);
        Title = "Native Sketch";
        AppWindow.Resize(new Windows.Graphics.SizeInt32(1100, 800));
        view.root.RowDefinitions.Add(new()
        {
            Height = GridLength.Auto
        });
        view.root.RowDefinitions.Add(new()
        {
            Height = GridLength.Auto
        });
        view.root.RowDefinitions.Add(new()
        {
            Height = new GridLength(1, GridUnitType.Star)
        });
        view.root.RowDefinitions.Add(new()
        {
            Height = GridLength.Auto
        });
        var menu = new MenuBar();
        var fm = new MenuBarItem()
        {
            Title = "File"
        };
        menu.Items.Add(fm);
        menus.Add(fm, "New", () => _ = documents.Document("new"));
        menus.Add(fm, "Open…", () => _ = documents.Document("open"));
        menus.Add(fm, "Save", () => _ = documents.Document("save"));
        menus.Add(fm, "Save As…", () => _ = documents.Document("saveAs"));
        var edit = new MenuBarItem()
        {
            Title = "Edit"
        };
        menu.Items.Add(edit);
        menus.Add(edit, "Undo", () => Command("undo"));
        menus.Add(edit, "Redo", () => Command("redo"));
        menus.Add(edit, "Delete selection", () => Command("delete"));
        menus.Add(edit, "Next stroke", () => Run(() =>
        {
            input.Cancel();
            view.select.IsChecked = true;
            engine.Cycle(1);
        }));
        menus.Add(edit, "Previous stroke", () => Run(() =>
        {
            input.Cancel();
            view.select.IsChecked = true;
            engine.Cycle(-1);
        }));
        var viewMenu = new MenuBarItem()
        {
            Title = "View"
        };
        menu.Items.Add(viewMenu);
        menus.Add(viewMenu, "Zoom in", () => navigation.Zoom(1.25));
        menus.Add(viewMenu, "Zoom out", () => navigation.Zoom(.8));
        menus.Add(viewMenu, "Reset view", () =>
        {
            input.Cancel();
            viewport.zoom = 1;
            viewport.panX = viewport.panY = 0;
            Draw();
        });
        view.root.Children.Add(menu);
        var bar = EditorToolbar.Build(view, engine, input.Cancel, Run, Command);
        Grid.SetRow(bar, 1);
        view.root.Children.Add(bar);
        Grid.SetRow(view.canvas, 2);
        view.root.Children.Add(view.canvas);
        Grid.SetRow(view.status, 3);
        view.root.Children.Add(view.status);
        Content = view.root;
        AutomationProperties.SetName(view.canvas, "Drawing canvas");
        AutomationProperties.SetName(view.width, "Brush width");
        AutomationProperties.SetName(view.scale, "Selection scale percent");
        view.canvas.PointerPressed += input.Pressed;
        view.canvas.PointerMoved += input.Moved;
        view.canvas.PointerReleased += input.Released;
        view.canvas.PointerCanceled += (_, _) => input.Cancel();
        view.canvas.PointerCaptureLost += (_, _) => input.Cancel();
        navigation.Attach();
        view.root.Loaded += (_, _) =>
        {
            view.root.XamlRoot.Changed += (_, _) => input.Cancel();
            Draw();
        };
        Activated += (_, e) =>
        {
            if (e.WindowActivationState == WindowActivationState.Deactivated)
            {
                input.space = false;
                input.Cancel();
            }
        };
        AppWindow.Closing += async (_, e) =>
        {
            if (documents.allowClose)
                return;
            e.Cancel = true;
            await documents.CloseSafely();
        };
        Closed += (_, _) => engine.Dispose();
    }

    void CancelInput() => input.Cancel();
    void Run(Action action)
    {
        if (documents.busy)
            return;
        try
        {
            action();
            Draw();
        }
        catch (Exception ex)
        {
            input.Cancel();
            _ = documents.Error(ex.Message);
        }
    }

    void Command(string name) => Run(() =>
    {
        input.Cancel();
        engine.Command(name);
    });
    void Draw()
    {
        painter?.Draw();
        try
        {
            presenter?.Refresh(engine.State);
        }
        catch (ObjectDisposedException)
        {
        }
    }
}
