using NativeSketch;
using System.Runtime.InteropServices;
static void Assert(bool ok, string message)
{
    if (!ok)
        throw new Exception(message);
}
Assert(Marshal.SizeOf<Sample>() == 48, "sample ABI size");
Assert(Marshal.SizeOf<Brush>() == 40, "brush ABI size");
Assert(Marshal.SizeOf<State>() == 64, "state ABI size");
Assert(Marshal.SizeOf<PathCommand>() == 96, "path ABI size");
using var e = new Engine();
var brush = new Brush()
{
    Width = 4,
    Alpha = 1
};
void Stroke(ulong id, bool pressure)
{
    e.Begin(id, pressure ? 1u : 0u, brush);
    e.Append(id, new[]
    {
        new Sample()
        {
            X = 10, Y = 20, Timestamp = 1, Id = 1,
            HasPressure = pressure ? 1u : 0u, Pressure = .1
        },
        new Sample()
        {
            X = 30, Y = 40, Timestamp = 2, Id = 2,
            HasPressure = pressure ? 1u : 0u, Pressure = 1
        }
    });
    e.Commit(id);
}
Stroke(1, false);
Assert(e.State.StrokeCount == 1 && e.Paths().Length > 0, "draw mouse");
e.Select(20, 30, 8);
Assert(e.State.SelectedId != 0, "hit test");
var original = e.State.Selection;
e.Command("resizeBegin");
e.Resize(2, 0, 0);
e.Command("cancel");
Assert(e.State.Selection.Width == original.Width, "resize cancellation");
e.Command("resizeBegin");
e.Resize(2, 0, 0);
e.Command("resizeCommit");
Assert(e.State.Selection.Width == 2 * original.Width, "resize");
e.Command("undo");
Assert(e.State.Selection.Width == original.Width, "undo");
e.Command("redo");
var dir = System.IO.Path.Combine(System.IO.Path.GetTempPath(), "sketch-interop-" + Guid.NewGuid());
Directory.CreateDirectory(dir);
try
{
    var file = System.IO.Path.Combine(dir, "unicode-рисунок.nsketch");
    e.Save(file);
    Assert(e.State.Dirty == 0, "save clean");
    Stroke(2, true);
    e.Save(file);
    Assert(e.State.StrokeCount == 2 && e.State.Dirty == 0, "replace save");
    var good = File.ReadAllBytes(file);
    Stroke(3, false);
    try
    {
        e.Save(dir);
        throw new Exception("Expected failed save");
    }
    catch (InvalidOperationException)
    {
    }
    Assert(e.State.Dirty == 1 && File.ReadAllBytes(file).SequenceEqual(good), "failed save preserves bytes/dirty");
    var corrupt = System.IO.Path.Combine(dir, "bad");
    File.WriteAllText(corrupt, "bad");
    try
    {
        e.Load(corrupt);
        throw new Exception("Expected failed load");
    }
    catch (InvalidOperationException)
    {
    }
    Assert(e.State.StrokeCount == 3, "corrupt load preserves document");
    e.Load(file);
    Assert(e.State.StrokeCount == 2 && e.State.Dirty == 0, "load");
    e.Cycle(1);
    e.Command("delete");
    e.Command("undo");
    Assert(e.State.StrokeCount == 2, "delete undo");
    e.Begin(99, 0, brush);
    e.Command("cancel");
    Assert(e.State.StrokeCount == 2, "ink cancel");
}
finally
{
    Directory.Delete(dir, true);
}
Console.WriteLine("PASS: C# / Rust ABI, mouse/pressure, paths, transforms, history, cancellation, Unicode repeated save, failed save/load.");
