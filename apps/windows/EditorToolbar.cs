using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace NativeSketch;

internal static class EditorToolbar
{
    internal static CommandBar Build(EditorView view, Engine engine, Action Cancel,
        Action<Action> Run, Action<string> Command)
    {
        var bar = new CommandBar()
        {
            DefaultLabelPosition = CommandBarDefaultLabelPosition.Right,
            IsDynamicOverflowEnabled = true
        };
        var tools = new StackPanel()
        {
            Orientation = Orientation.Horizontal,
            Spacing = 12,
            VerticalAlignment = VerticalAlignment.Center
        };
        tools.Children.Add(view.pen);
        tools.Children.Add(view.select);
        bar.Content = tools;
        view.pen.Checked += (_, _) =>
        {
            Cancel();
            Run(() => engine.Command("clear"));
        };
        view.select.Checked += (_, _) => Cancel();
        bar.PrimaryCommands.Add(view.undo);
        bar.PrimaryCommands.Add(view.redo);
        view.undo.Click += (_, _) => Command("undo");
        view.redo.Click += (_, _) => Command("redo");
        var style = new AppBarButton()
        {
            Label = "Brush",
            Icon = new SymbolIcon(Symbol.Edit)
        };
        var settings = new StackPanel()
        {
            Spacing = 12
        };
        settings.Children.Add(view.width);
        settings.Children.Add(view.color);
        style.Flyout = new Flyout()
        {
            Content = settings
        };
        bar.PrimaryCommands.Add(style);
        var transform = new AppBarButton()
        {
            Label = "Scale selection"
        };
        var ts = new StackPanel()
        {
            Spacing = 8
        };
        ts.Children.Add(view.scale);
        ts.Children.Add(view.apply);
        transform.Flyout = new Flyout()
        {
            Content = ts
        };
        bar.PrimaryCommands.Add(transform);
        view.apply.Click += (_, _) => Run(() =>
        {
            Cancel();
            var b = engine.State.Selection;
            if (!double.IsFinite(view.scale.Value) || view.scale.Value <= 0)
                throw new ArgumentException("Enter a positive scale");
            engine.Command("resizeBegin");
            engine.Resize(view.scale.Value / 100, b.X + b.Width / 2, b.Y + b.Height / 2);
            engine.Command("resizeCommit");
        });
        return bar;
    }
}
