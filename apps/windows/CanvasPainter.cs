using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Windows.UI;
using Path = Microsoft.UI.Xaml.Shapes.Path;

namespace NativeSketch;

internal sealed class CanvasPainter
{
    readonly Engine engine;
    readonly EditorView view;
    readonly CanvasViewport viewport;
    internal CanvasPainter(Engine engine, EditorView view, CanvasViewport viewport)
    {
        this.engine = engine;
        this.view = view;
        this.viewport = viewport;
    }

    internal void Draw()
    {
        if (view.canvas == null)
            return;
        view.canvas.Children.Clear();
        State s;
        try
        {
            s = engine.State;
        }
        catch (ObjectDisposedException)
        {
            return;
        }
        foreach (var group in engine.Paths().GroupBy(c => c.StrokeId))
        {
            var g = new PathGeometry()
            {
                FillRule = FillRule.Nonzero
            };
            PathFigure? figure = null;
            var first = group.First();
            foreach (var c in group)
            {
                switch (c.Verb)
                {
                    case 0:
                        figure = new PathFigure()
                        {
                            StartPoint = new(c.X1, c.Y1),
                            IsFilled = true
                        };
                        g.Figures.Add(figure);
                        break;
                    case 1:
                        figure!.Segments.Add(new LineSegment()
                        {
                            Point = new(c.X1, c.Y1)
                        });
                        break;
                    case 2:
                        figure!.Segments.Add(new BezierSegment()
                        {
                            Point1 = new(c.X1, c.Y1),
                            Point2 = new(c.X2, c.Y2),
                            Point3 = new(c.X3, c.Y3)
                        });
                        break;
                    case 3:
                        figure!.IsClosed = true;
                        break;
                }
            }
            var path = new Path()
            {
                Data = g,
                Fill = new SolidColorBrush(Color.FromArgb((byte)(first.Alpha * 255), (byte)(first.Red * 255), (byte)(first.Green * 255), (byte)(first.Blue * 255))),
                IsHitTestVisible = false,
                RenderTransform = new MatrixTransform()
                {
                    Matrix = new Matrix(viewport.zoom, 0, 0, viewport.zoom, viewport.panX, viewport.panY)
                }
            };
            view.canvas.Children.Add(path);
        }
        if (s.SelectedId != 0)
        {
            var b = s.Selection;
            var r = new Microsoft.UI.Xaml.Shapes.Rectangle()
            {
                Width = b.Width * viewport.zoom,
                Height = b.Height * viewport.zoom,
                Stroke = new SolidColorBrush(Microsoft.UI.Colors.DodgerBlue),
                StrokeThickness = 1,
                IsHitTestVisible = false
            };
            Canvas.SetLeft(r, b.X * viewport.zoom + viewport.panX);
            Canvas.SetTop(r, b.Y * viewport.zoom + viewport.panY);
            view.canvas.Children.Add(r);
            foreach (var x in new[] { b.X, b.X + b.Width })
                foreach (var y in new[] { b.Y, b.Y + b.Height })
                {
                    var h = new Microsoft.UI.Xaml.Shapes.Rectangle()
                    {
                        Width = 8,
                        Height = 8,
                        Fill = new SolidColorBrush(Microsoft.UI.Colors.White),
                        Stroke = new SolidColorBrush(Microsoft.UI.Colors.DodgerBlue),
                        StrokeThickness = 1,
                        IsHitTestVisible = false
                    };
                    Canvas.SetLeft(h, x * viewport.zoom + viewport.panX - 4);
                    Canvas.SetTop(h, y * viewport.zoom + viewport.panY - 4);
                    view.canvas.Children.Add(h);
                }
        }
    }
}
