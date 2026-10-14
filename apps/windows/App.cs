using Microsoft.UI;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Input;
using Windows.Foundation;
using Windows.System;
using Windows.Storage.Pickers;
using Windows.UI;
using Path = Microsoft.UI.Xaml.Shapes.Path;
namespace NativeSketch;

public sealed class App : Application
{
    [STAThread]
    public static void Main()
    {
        WinRT.ComWrappersSupport.InitializeComWrappers();
        Application.Start(_ => { SynchronizationContext.SetSynchronizationContext(new Microsoft.UI.Dispatching.DispatcherQueueSynchronizationContext(Microsoft.UI.Dispatching.DispatcherQueue.GetForCurrentThread())); new App(); });
    }
    public App()
    {
        Resources.MergedDictionaries.Add(new XamlControlsResources());
    }
    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        new SketchWindow().Activate();
    }
}
public sealed class SketchWindow : Window
{
    readonly Dictionary<string, MenuFlyoutItem> menuItems = new();
    readonly Engine engine = new(); readonly Grid root = new(); readonly Canvas canvas = new() { Background = new SolidColorBrush(Microsoft.UI.Colors.White), IsTabStop = true };
    readonly TextBlock status = new() { Margin = new Thickness(12, 4, 12, 4) };
    readonly NumberBox width = new() { Header = "Width", Value = 4, Minimum = .1, Maximum = 256, Width = 105, SpinButtonPlacementMode = NumberBoxSpinButtonPlacementMode.Compact };
    readonly NumberBox scale = new() { Header = "Scale %", Value = 100, Minimum = 1, Maximum = 10000, Width = 110 };
    readonly ColorPicker color = new() { Color = Microsoft.UI.Colors.Black, IsAlphaEnabled = true };
    readonly AppBarButton undo = new() { Label = "Undo", Icon = new SymbolIcon(Symbol.Undo) }, redo = new() { Label = "Redo", Icon = new SymbolIcon(Symbol.Redo) }, apply = new() { Label = "Apply scale" };
    readonly RadioButton pen = new() { Content = "Pen", IsChecked = true }, select = new() { Content = "Select" };
    double zoom = 1, panX, panY, oldWidth, oldHeight; ulong lastFrame; uint? pointer; bool panning, resizing, space, busy, allowClose; Point start, anchor; double radius; string? file;
    public SketchWindow()
    {
        Title = "Native Sketch";
        AppWindow.Resize(new Windows.Graphics.SizeInt32(1100, 800));
        root.RowDefinitions.Add(new()
        {
            Height = GridLength.Auto
        });
        root.RowDefinitions.Add(new()
        {
            Height = GridLength.Auto
        });
        root.RowDefinitions.Add(new()
        {
            Height = new GridLength(1, GridUnitType.Star)
        });
        root.RowDefinitions.Add(new()
        {
            Height = GridLength.Auto
        });
        var menu = new MenuBar();
        var fm = new MenuBarItem() { Title = "File" };
        menu.Items.Add(fm);
        Menu(fm, "New", () => _ = Document("new"));
        Menu(fm, "Open…", () => _ = Document("open"));
        Menu(fm, "Save", () => _ = Document("save"));
        Menu(fm, "Save As…", () => _ = Document("saveAs"));
        var edit = new MenuBarItem() { Title = "Edit" };
        menu.Items.Add(edit);
        Menu(edit, "Undo", () => Command("undo"));
        Menu(edit, "Redo", () => Command("redo"));
        Menu(edit, "Delete selection", () => Command("delete"));
        Menu(edit, "Next stroke", () => Run(() => { Cancel(); select.IsChecked = true; engine.Cycle(1); }));
        Menu(edit, "Previous stroke", () => Run(() => { Cancel(); select.IsChecked = true; engine.Cycle(-1); }));
        var view = new MenuBarItem() { Title = "View" };
        menu.Items.Add(view);
        Menu(view, "Zoom in", () => Zoom(1.25));
        Menu(view, "Zoom out", () => Zoom(.8));
        Menu(view, "Reset view", () => { Cancel(); zoom = 1; panX = panY = 0; Draw(); });
        root.Children.Add(menu);
        var bar = new CommandBar() { DefaultLabelPosition = CommandBarDefaultLabelPosition.Right, IsDynamicOverflowEnabled = true };
        var tools = new StackPanel() { Orientation = Orientation.Horizontal, Spacing = 12, VerticalAlignment = VerticalAlignment.Center };
        tools.Children.Add(pen);
        tools.Children.Add(select);
        bar.Content = tools;
        pen.Checked += (_, _) => { Cancel(); Run(() => engine.Command("clear")); };
        select.Checked += (_, _) => Cancel();
        bar.PrimaryCommands.Add(undo);
        bar.PrimaryCommands.Add(redo);
        undo.Click += (_, _) => Command("undo");
        redo.Click += (_, _) => Command("redo");
        var style = new AppBarButton() { Label = "Brush", Icon = new SymbolIcon(Symbol.Edit) };
        var settings = new StackPanel() { Spacing = 12 };
        settings.Children.Add(width);
        settings.Children.Add(color);
        style.Flyout = new Flyout() { Content = settings };
        bar.PrimaryCommands.Add(style);
        var transform = new AppBarButton() { Label = "Scale selection" };
        var ts = new StackPanel() { Spacing = 8 };
        ts.Children.Add(scale);
        ts.Children.Add(apply);
        transform.Flyout = new Flyout() { Content = ts };
        bar.PrimaryCommands.Add(transform);
        apply.Click += (_, _) => Run(() => { Cancel(); var b = engine.State.Selection; if (!double.IsFinite(scale.Value) || scale.Value <= 0) throw new ArgumentException("Enter a positive scale"); engine.Command("resizeBegin"); engine.Resize(scale.Value / 100, b.X + b.Width / 2, b.Y + b.Height / 2); engine.Command("resizeCommit"); });
        Grid.SetRow(bar, 1);
        root.Children.Add(bar);
        Grid.SetRow(canvas, 2);
        root.Children.Add(canvas);
        Grid.SetRow(status, 3);
        root.Children.Add(status);
        Content = root;
        AutomationProperties.SetName(canvas, "Drawing canvas");
        AutomationProperties.SetName(width, "Brush width");
        AutomationProperties.SetName(scale, "Selection scale percent");
        canvas.PointerPressed += Pressed;
        canvas.PointerMoved += Moved;
        canvas.PointerReleased += Released;
        canvas.PointerCanceled += (_, _) => Cancel();
        canvas.PointerCaptureLost += (_, _) => Cancel();
        canvas.PointerWheelChanged += (_, e) => { if (pointer != null) return; var p = e.GetCurrentPoint(canvas); if ((e.KeyModifiers & VirtualKeyModifiers.Control) != 0) Zoom(p.Properties.MouseWheelDelta > 0 ? 1.1 : 1 / 1.1); else { if (p.Properties.IsHorizontalMouseWheel) panX += p.Properties.MouseWheelDelta / 3.0; else panY += p.Properties.MouseWheelDelta / 3.0; Draw(); } e.Handled = true; };
        canvas.KeyDown += Key;
        canvas.KeyUp += (_, e) => { if (e.Key == VirtualKey.Space) space = false; };
        canvas.LostFocus += (_, _) => { space = false; Cancel(); };
        canvas.SizeChanged += (_, e) => { Cancel(); if (oldWidth > 0) { panX += (e.NewSize.Width - oldWidth) / 2; panY += (e.NewSize.Height - oldHeight) / 2; } oldWidth = e.NewSize.Width; oldHeight = e.NewSize.Height; Draw(); };
        root.Loaded += (_, _) => { root.XamlRoot.Changed += (_, _) => Cancel(); Draw(); };
        Activated += (_, e) => { if (e.WindowActivationState == WindowActivationState.Deactivated) { space = false; Cancel(); } };
        AppWindow.Closing += async (_, e) => { if (allowClose) return; e.Cancel = true; await CloseSafely(); };
        Closed += (_, _) => engine.Dispose();
    }
    void Menu(MenuBarItem parent, string text, Action action)
    {
        var item = new MenuFlyoutItem() { Text = text };
        item.Click += (_, _) => { if (!busy) action(); };
        var key = text switch
        {
            "New" => VirtualKey.N,
            "Open…" => VirtualKey.O,
            "Save" => VirtualKey.S,
            "Save As…" => VirtualKey.S,
            "Undo" => VirtualKey.Z,
            "Redo" => VirtualKey.Y,
            _ => VirtualKey.None
        };
        if (key != VirtualKey.None)
        {
            var accelerator = new KeyboardAccelerator() { Key = key, Modifiers = VirtualKeyModifiers.Control | (text == "Save As…" ? VirtualKeyModifiers.Shift : VirtualKeyModifiers.None) };
            accelerator.Invoked += (_, e) => { if ((text == "Undo" || text == "Redo") && FocusManager.GetFocusedElement(root.XamlRoot) is TextBox) return; if (!busy && item.IsEnabled) action(); e.Handled = true; };
            item.KeyboardAccelerators.Add(accelerator);
        }
        parent.Items.Add(item);
        menuItems.Add(text, item);
    }
    void Run(Action action)
    {
        if (busy)
            return;
        try
        {
            action();
            Draw();
        }
        catch (Exception ex) { Cancel(); _ = Error(ex.Message); }
    }
    void Command(string name) => Run(() => { Cancel(); engine.Command(name); });
    void Cancel()
    {
        pointer = null;
        panning = resizing = false;
        canvas.ReleasePointerCaptures();
        try
        {
            engine.Command("cancel");
        }
        catch (ObjectDisposedException) { }
        Draw();
    }
    Point Doc(Point p) => new((p.X - panX) / zoom, (p.Y - panY) / zoom);
    void Pressed(object sender, PointerRoutedEventArgs e)
    {
        if (busy)
            return;
        if (pointer != null)
        {
            Cancel();
            return;
        }
        var p = e.GetCurrentPoint(canvas);
        if (!p.Properties.IsLeftButtonPressed && p.PointerDeviceType == PointerDeviceType.Mouse)
            return;
        canvas.Focus(FocusState.Pointer);
        Run(() =>
        {
            start = p.Position;
            panning = space;
            if (panning)
            {
            }
            else if (select.IsChecked == true)
            {
                var b = engine.State.Selection;
                var corners = new[] { new Point(b.X, b.Y), new Point(b.X + b.Width, b.Y), new Point(b.X + b.Width, b.Y + b.Height), new Point(b.X, b.Y + b.Height) };
                int index = engine.State.SelectedId == 0 ? -1 : Array.FindIndex(corners, c => Math.Abs(c.X * zoom + panX - start.X) <= 10 && Math.Abs(c.Y * zoom + panY - start.Y) <= 10);
                if (index >= 0)
                {
                    resizing = true;
                    anchor = corners[(index + 2) % 4];
                    var d = Doc(start);
                    radius = Math.Max(.001, Distance(d, anchor));
                    engine.Command("resizeBegin");
                }
                else
                {
                    var d = Doc(start);
                    engine.Select(d.X, d.Y, 8 / zoom);
                }
            }
            else
            {
                if (!double.IsFinite(width.Value) || width.Value <= 0)
                    throw new ArgumentException("Enter a positive brush width");
                var c = color.Color;
                engine.Command("clear");
                engine.Begin(p.PointerId, p.PointerDeviceType == PointerDeviceType.Pen ? 1u : p.PointerDeviceType == PointerDeviceType.Touch ? 2u : 0u, new Brush() { Red = c.R / 255.0, Green = c.G / 255.0, Blue = c.B / 255.0, Alpha = c.A / 255.0, Width = width.Value });
            }
            lastFrame = 0;
            pointer = p.PointerId;
            if (!canvas.CapturePointer(e.Pointer))
            {
                Cancel();
                return;
            }
            if (!panning && !resizing && pen.IsChecked == true)
                Append(e);
        });
        e.Handled = true;
    }
    static double Distance(Point a, Point b) => Math.Sqrt((a.X - b.X) * (a.X - b.X) + (a.Y - b.Y) * (a.Y - b.Y));
    void Append(PointerRoutedEventArgs e)
    {
        var points = e.GetIntermediatePoints(canvas).GroupBy(p => p.FrameId).Select(g => g.Last()).OrderBy(p => p.FrameId).Where(p => p.FrameId > lastFrame).Select(p => { var d = Doc(p.Position); return new Sample() { X = d.X, Y = d.Y, Timestamp = p.Timestamp / 1000000.0, Pressure = p.Properties.Pressure, HasPressure = p.PointerDeviceType == PointerDeviceType.Pen ? 1u : 0u, Id = p.FrameId }; }).ToArray();
        if (points.Length > 0)
        {
            engine.Append(pointer!.Value, points);
            lastFrame = points[^1].Id;
        }
    }
    void Moved(object sender, PointerRoutedEventArgs e)
    {
        if (pointer != e.Pointer.PointerId)
            return;
        Run(() => { var p = e.GetCurrentPoint(canvas).Position; if (panning) { panX += p.X - start.X; panY += p.Y - start.Y; start = p; } else if (resizing) engine.Resize(Math.Max(.01, Distance(Doc(p), anchor) / radius), anchor.X, anchor.Y); else if (pen.IsChecked == true) Append(e); });
        e.Handled = true;
    }
    void Released(object sender, PointerRoutedEventArgs e)
    {
        if (pointer != e.Pointer.PointerId)
            return;
        Run(() => { if (resizing) { engine.Resize(Math.Max(.01, Distance(Doc(e.GetCurrentPoint(canvas).Position), anchor) / radius), anchor.X, anchor.Y); engine.Command("resizeCommit"); } else if (!panning && pen.IsChecked == true) { Append(e); engine.Commit(pointer.Value); } pointer = null; resizing = panning = false; canvas.ReleasePointerCaptures(); });
        e.Handled = true;
    }
    void Key(object sender, KeyRoutedEventArgs e)
    {
        if (!ReferenceEquals(e.OriginalSource, canvas))
            return;
        switch (e.Key)
        {
            case VirtualKey.Escape:
                Cancel();
                break;
            case VirtualKey.Space:
                space = true;
                break;
            case VirtualKey.P:
                pen.IsChecked = true;
                break;
            case VirtualKey.V:
                select.IsChecked = true;
                break;
            case VirtualKey.Delete:
                Command("delete");
                break;
            default:
                return;
        }
        e.Handled = true;
    }
    void Zoom(double factor)
    {
        if (pointer != null)
            return;
        double next = Math.Clamp(zoom * factor, .05, 32);
        var center = new Point(canvas.ActualWidth / 2, canvas.ActualHeight / 2);
        var d = Doc(center);
        zoom = next;
        panX = center.X - d.X * zoom;
        panY = center.Y - d.Y * zoom;
        Draw();
    }
    void Draw()
    {
        if (canvas == null)
            return;
        canvas.Children.Clear();
        State s;
        try
        {
            s = engine.State;
        }
        catch (ObjectDisposedException) { return; }
        foreach (var group in engine.Paths().GroupBy(c => c.StrokeId))
        {
            var g = new PathGeometry() { FillRule = FillRule.Nonzero };
            PathFigure? figure = null;
            var first = group.First();
            foreach (var c in group)
            {
                switch (c.Verb)
                {
                    case 0:
                        figure = new PathFigure() { StartPoint = new(c.X1, c.Y1), IsFilled = true };
                        g.Figures.Add(figure);
                        break;
                    case 1:
                        figure!.Segments.Add(new LineSegment() { Point = new(c.X1, c.Y1) });
                        break;
                    case 2:
                        figure!.Segments.Add(new BezierSegment() { Point1 = new(c.X1, c.Y1), Point2 = new(c.X2, c.Y2), Point3 = new(c.X3, c.Y3) });
                        break;
                    case 3:
                        figure!.IsClosed = true;
                        break;
                }
            }
            var path = new Path() { Data = g, Fill = new SolidColorBrush(Color.FromArgb((byte)(first.Alpha * 255), (byte)(first.Red * 255), (byte)(first.Green * 255), (byte)(first.Blue * 255))), IsHitTestVisible = false, RenderTransform = new MatrixTransform() { Matrix = new Matrix(zoom, 0, 0, zoom, panX, panY) } };
            canvas.Children.Add(path);
        }
        if (s.SelectedId != 0)
        {
            var b = s.Selection;
            var r = new Microsoft.UI.Xaml.Shapes.Rectangle() { Width = b.Width * zoom, Height = b.Height * zoom, Stroke = new SolidColorBrush(Microsoft.UI.Colors.DodgerBlue), StrokeThickness = 1, IsHitTestVisible = false };
            Canvas.SetLeft(r, b.X * zoom + panX);
            Canvas.SetTop(r, b.Y * zoom + panY);
            canvas.Children.Add(r);
            foreach (var x in new[] { b.X, b.X + b.Width })
                foreach (var y in new[] { b.Y, b.Y + b.Height })
                {
                    var h = new Microsoft.UI.Xaml.Shapes.Rectangle() { Width = 8, Height = 8, Fill = new SolidColorBrush(Microsoft.UI.Colors.White), Stroke = new SolidColorBrush(Microsoft.UI.Colors.DodgerBlue), StrokeThickness = 1, IsHitTestVisible = false };
                    Canvas.SetLeft(h, x * zoom + panX - 4);
                    Canvas.SetTop(h, y * zoom + panY - 4);
                    canvas.Children.Add(h);
                }
        }
        status.Text = $"{(s.Dirty != 0 ? "Unsaved · " : "")}{s.StrokeCount} strokes · selected {s.SelectedId} · {zoom:P0}" + (s.StrokeCount == 0 ? " · Draw with Pen; Space-drag to pan" : "");
        AutomationProperties.SetName(canvas, "Drawing canvas. " + status.Text);
        foreach (var item in menuItems.Values) item.IsEnabled = !busy;
        undo.IsEnabled = menuItems["Undo"].IsEnabled = !busy && s.CanUndo != 0;
        redo.IsEnabled = menuItems["Redo"].IsEnabled = !busy && s.CanRedo != 0;
        apply.IsEnabled = menuItems["Delete selection"].IsEnabled = !busy && s.SelectedId != 0;
        menuItems["Next stroke"].IsEnabled = menuItems["Previous stroke"].IsEnabled = !busy && s.StrokeCount != 0;
        Title = (file == null ? "Untitled" : System.IO.Path.GetFileName(file)) + (s.Dirty != 0 ? " *" : "") + " — Native Sketch";
    }
    async Task Error(string message)
    {
        await new ContentDialog() { Title = "Native Sketch", Content = message, CloseButtonText = "OK", XamlRoot = root.XamlRoot }.ShowAsync();
    }
    async Task<bool> Save(bool choose)
    {
        string? path = file;
        if (path == null || choose)
        {
            var picker = new FileSavePicker() { SuggestedFileName = "Sketch" };
            picker.FileTypeChoices.Add("Native Sketch", new List<string> { ".nsketch" });
            WinRT.Interop.InitializeWithWindow.Initialize(picker, WinRT.Interop.WindowNative.GetWindowHandle(this));
            var target = await picker.PickSaveFileAsync();
            if (target == null)
                return false;
            path = target.Path;
        }
        engine.Save(path);
        file = path;
        return true;
    }
    async Task<bool> Confirm()
    {
        if (engine.State.Dirty == 0)
            return true;
        var result = await new ContentDialog() { Title = "Save changes?", Content = "This drawing has unsaved changes.", PrimaryButtonText = "Save", SecondaryButtonText = "Discard", CloseButtonText = "Cancel", DefaultButton = ContentDialogButton.Primary, XamlRoot = root.XamlRoot }.ShowAsync();
        return result == ContentDialogResult.Secondary || (result == ContentDialogResult.Primary && await Save(false));
    }
    async Task Document(string operation)
    {
        if (busy)
            return;
        Cancel();
        busy = true;
        canvas.IsHitTestVisible = false;
        Draw();
        try
        {
            if (operation == "save" || operation == "saveAs")
                await Save(operation == "saveAs");
            else if (await Confirm())
            {
                if (operation == "new")
                {
                    engine.Command("new");
                    file = null;
                }
                else
                {
                    var picker = new FileOpenPicker();
                    picker.FileTypeFilter.Add(".nsketch");
                    WinRT.Interop.InitializeWithWindow.Initialize(picker, WinRT.Interop.WindowNative.GetWindowHandle(this));
                    var target = await picker.PickSingleFileAsync();
                    if (target != null)
                    {
                        engine.Load(target.Path);
                        file = target.Path;
                    }
                }
            }
        }
        catch (Exception ex) { await Error(ex.Message); }
        finally { busy = false; canvas.IsHitTestVisible = true; Draw(); }
    }
    async Task CloseSafely()
    {
        if (busy)
            return;
        Cancel();
        busy = true;
        try
        {
            if (await Confirm())
            {
                allowClose = true;
                Close();
            }
        }
        catch (Exception ex) { await Error(ex.Message); }
        finally { busy = false; }
    }
}
