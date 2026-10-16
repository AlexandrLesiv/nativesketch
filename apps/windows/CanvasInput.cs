using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Input;
using Windows.Foundation;

namespace NativeSketch;

internal sealed class CanvasInput
{
    readonly Engine engine;
    readonly EditorView view;
    readonly CanvasViewport viewport;
    readonly Action<Action> Run;
    readonly Action Draw;
    readonly Func<bool> isBusy;
    bool busy => isBusy();
    ulong lastFrame;
    internal uint? pointer;
    bool panning, resizing;
    internal bool space;
    Point start, anchor;
    double radius;
    internal CanvasInput(Engine engine, EditorView view, CanvasViewport viewport,
        Action<Action> run, Action draw, Func<bool> isBusy)
    {
        this.engine = engine;
        this.view = view;
        this.viewport = viewport;
        Run = run;
        Draw = draw;
        this.isBusy = isBusy;
    }

    internal void Cancel()
    {
        pointer = null;
        panning = resizing = false;
        view.canvas.ReleasePointerCaptures();
        try
        {
            engine.Command("cancel");
        }
        catch (ObjectDisposedException)
        {
        }
        Draw();
    }

    internal void Pressed(object sender, PointerRoutedEventArgs e)
    {
        if (busy)
            return;
        if (pointer != null)
        {
            Cancel();
            return;
        }
        var p = e.GetCurrentPoint(view.canvas);
        if (!p.Properties.IsLeftButtonPressed && p.PointerDeviceType == PointerDeviceType.Mouse)
            return;
        view.canvas.Focus(FocusState.Pointer);
        Run(() =>
        {
            start = p.Position;
            panning = space;
            if (panning)
            {
            }
            else if (view.select.IsChecked == true)
            {
                var b = engine.State.Selection;
                var corners = new[]
                {
                    new Point(b.X, b.Y),
                    new Point(b.X + b.Width, b.Y),
                    new Point(b.X + b.Width, b.Y + b.Height),
                    new Point(b.X, b.Y + b.Height)
                };
                int index = engine.State.SelectedId == 0 ? -1 : Array.FindIndex(corners, c => Math.Abs(c.X * viewport.zoom + viewport.panX - start.X) <= 10 && Math.Abs(c.Y * viewport.zoom + viewport.panY - start.Y) <= 10);
                if (index >= 0)
                {
                    resizing = true;
                    anchor = corners[(index + 2) % 4];
                    var d = viewport.Doc(start);
                    radius = Math.Max(.001, Distance(d, anchor));
                    engine.Command("resizeBegin");
                }
                else
                {
                    var d = viewport.Doc(start);
                    engine.Select(d.X, d.Y, 8 / viewport.zoom);
                }
            }
            else
            {
                if (!double.IsFinite(view.width.Value) || view.width.Value <= 0)
                    throw new ArgumentException("Enter a positive brush width");
                var c = view.color.Color;
                engine.Command("clear");
                engine.Begin(p.PointerId, p.PointerDeviceType == PointerDeviceType.Pen ? 1u : p.PointerDeviceType == PointerDeviceType.Touch ? 2u : 0u, new Brush()
                {
                    Red = c.R / 255.0,
                    Green = c.G / 255.0,
                    Blue = c.B / 255.0,
                    Alpha = c.A / 255.0,
                    Width = view.width.Value
                });
            }
            lastFrame = 0;
            pointer = p.PointerId;
            if (!view.canvas.CapturePointer(e.Pointer))
            {
                Cancel();
                return;
            }
            if (!panning && !resizing && view.pen.IsChecked == true)
                Append(e);
        });
        e.Handled = true;
    }

    static double Distance(Point a, Point b) => Math.Sqrt((a.X - b.X) * (a.X - b.X) + (a.Y - b.Y) * (a.Y - b.Y));
    void Append(PointerRoutedEventArgs e)
    {
        var points = e.GetIntermediatePoints(view.canvas).GroupBy(p => p.FrameId).Select(g => g.Last()).OrderBy(p => p.FrameId).Where(p => p.FrameId > lastFrame).Select(p =>
        {
            var d = viewport.Doc(p.Position);
            return new Sample()
            {
                X = d.X,
                Y = d.Y,
                Timestamp = p.Timestamp / 1000000.0,
                Pressure = p.Properties.Pressure,
                HasPressure = p.PointerDeviceType == PointerDeviceType.Pen ? 1u : 0u,
                Id = p.FrameId
            };
        }).ToArray();
        if (points.Length > 0)
        {
            engine.Append(pointer!.Value, points);
            lastFrame = points[^1].Id;
        }
    }

    internal void Moved(object sender, PointerRoutedEventArgs e)
    {
        if (pointer != e.Pointer.PointerId)
            return;
        Run(() =>
        {
            var p = e.GetCurrentPoint(view.canvas).Position;
            if (panning)
            {
                viewport.panX += p.X - start.X;
                viewport.panY += p.Y - start.Y;
                start = p;
            }
            else if (resizing)
                engine.Resize(Math.Max(.01, Distance(viewport.Doc(p), anchor) / radius), anchor.X, anchor.Y);
            else if (view.pen.IsChecked == true)
                Append(e);
        });
        e.Handled = true;
    }

    internal void Released(object sender, PointerRoutedEventArgs e)
    {
        if (pointer != e.Pointer.PointerId)
            return;
        Run(() =>
        {
            if (resizing)
            {
                engine.Resize(Math.Max(.01, Distance(viewport.Doc(e.GetCurrentPoint(view.canvas).Position), anchor) / radius), anchor.X, anchor.Y);
                engine.Command("resizeCommit");
            }
            else if (!panning && view.pen.IsChecked == true)
            {
                Append(e);
                engine.Commit(pointer.Value);
            }
            pointer = null;
            resizing = panning = false;
            view.canvas.ReleasePointerCaptures();
        });
        e.Handled = true;
    }
}
