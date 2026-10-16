using Windows.Foundation;

namespace NativeSketch;

internal sealed class CanvasViewport
{
    internal double zoom = 1, panX, panY, oldWidth, oldHeight;
    internal Point Doc(Point p) => new((p.X - panX) / zoom, (p.Y - panY) / zoom);
}
