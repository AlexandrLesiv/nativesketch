using Microsoft.UI.Xaml.Input;
using Windows.Foundation;
using Windows.System;

namespace NativeSketch;

internal sealed class CanvasNavigation
{
    readonly Engine engine;
    readonly EditorView view;
    readonly CanvasViewport viewport;
    readonly CanvasInput input;
    readonly Action Draw;
    readonly Action<string> Command;
    internal CanvasNavigation(Engine engine, EditorView view, CanvasViewport viewport,
        CanvasInput input, Action draw, Action<string> command)
    {
        this.engine = engine;
        this.view = view;
        this.viewport = viewport;
        this.input = input;
        Draw = draw;
        Command = command;
    }

    internal void Key(object sender, KeyRoutedEventArgs e)
    {
        if (!ReferenceEquals(e.OriginalSource, view.canvas))
            return;
        switch (e.Key)
        {
            case VirtualKey.Escape:
                input.Cancel();
                break;
            case VirtualKey.Space:
                input.space = true;
                break;
            case VirtualKey.P:
                view.pen.IsChecked = true;
                break;
            case VirtualKey.V:
                view.select.IsChecked = true;
                break;
            case VirtualKey.Delete:
                if (engine.State.SelectedId != 0)
                    Command("delete");
                break;
            default:
                return;
        }
        e.Handled = true;
    }

    internal void Zoom(double factor)
    {
        if (input.pointer != null)
            return;
        double next = Math.Clamp(viewport.zoom * factor, .05, 32);
        var center = new Point(view.canvas.ActualWidth / 2, view.canvas.ActualHeight / 2);
        var d = viewport.Doc(center);
        viewport.zoom = next;
        viewport.panX = center.X - d.X * viewport.zoom;
        viewport.panY = center.Y - d.Y * viewport.zoom;
        Draw();
    }

    internal void Attach()
    {
        view.canvas.PointerWheelChanged += (_, e) =>
        {
            if (input.pointer != null)
                return;
            var p = e.GetCurrentPoint(view.canvas);
            if ((e.KeyModifiers & VirtualKeyModifiers.Control) != 0)
                Zoom(p.Properties.MouseWheelDelta > 0 ? 1.1 : 1 / 1.1);
            else
            {
                if (p.Properties.IsHorizontalMouseWheel)
                    viewport.panX += p.Properties.MouseWheelDelta / 3.0;
                else
                    viewport.panY += p.Properties.MouseWheelDelta / 3.0;
                Draw();
            }
            e.Handled = true;
        };
        view.canvas.KeyDown += Key;
        view.canvas.KeyUp += (_, e) =>
        {
            if (e.Key == VirtualKey.Space)
                input.space = false;
        };
        view.canvas.LostFocus += (_, _) =>
        {
            input.space = false;
            input.Cancel();
        };
        view.canvas.SizeChanged += (_, e) =>
        {
            input.Cancel();
            if (viewport.oldWidth > 0)
            {
                viewport.panX += (e.NewSize.Width - viewport.oldWidth) / 2;
                viewport.panY += (e.NewSize.Height - viewport.oldHeight) / 2;
            }
            viewport.oldWidth = e.NewSize.Width;
            viewport.oldHeight = e.NewSize.Height;
            Draw();
        };
    }
}
