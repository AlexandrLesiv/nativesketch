using System.Runtime.InteropServices;
using System.Text;
namespace NativeSketch;

[StructLayout(LayoutKind.Sequential)]
public struct Sample
{
    public double X, Y, Timestamp, Pressure;
    public ulong Id;
    public uint HasPressure, Kind;
}
[StructLayout(LayoutKind.Sequential)]
public struct Brush
{
    public double Red, Green, Blue, Alpha, Width;
}
[StructLayout(LayoutKind.Sequential)]
public struct Bounds
{
    public double X, Y, Width, Height;
}
[StructLayout(LayoutKind.Sequential)]
public struct State
{
    public ulong StrokeCount, SelectedId;
    public uint CanUndo, CanRedo, Dirty, Interaction;
    public Bounds Selection;
}
[StructLayout(LayoutKind.Sequential)]
public struct PathCommand
{
    public ulong StrokeId;
    public uint Verb;
    public double Red, Green, Blue, Alpha, X1, Y1, X2, Y2, X3, Y3;
}
// One unique handle; all calls, including Dispose, are serialized on its creating thread.
// No native callbacks. Input arrays are borrowed during a synchronous call only.
public sealed class Engine : IDisposable
{
    IntPtr handle;
    readonly int owner = Environment.CurrentManagedThreadId;
    public Engine()
    {
        if (N.sketch_abi_version() != 2)
            throw new InvalidOperationException("Unsupported Rust ABI");
        Check(N.sketch_editor_create(out handle));
    }

    void Guard()
    {
        if (owner != Environment.CurrentManagedThreadId)
            throw new InvalidOperationException("Editor thread ownership violated");
        ObjectDisposedException.ThrowIf(handle == IntPtr.Zero, this);
    }

    void Check(uint code)
    {
        if (code == 6)
            Dispose();
        if (code != 0)
            throw new InvalidOperationException($"Rust editor error {code}: " + (code switch
            {
                1 => "Invalid argument",
                2 => "Invalid editor state",
                3 => "Insufficient capacity",
                4 => "File operation failed",
                5 => "Invalid document",
                6 => "Engine panic; editor discarded",
                _ => "Unknown status"
            }));
    }
    public State State
    {
        get
        {
            Guard();
            Check(N.sketch_editor_state(handle, out var value));
            return value;
        }
    }

    public void Begin(ulong id, uint device, Brush brush)
    {
        Guard();
        Check(N.sketch_stroke_begin(handle, id, device, brush));
    }

    public void Append(ulong id, Sample[] samples)
    {
        Guard();
        Check(N.sketch_stroke_append(handle, id, samples, (nuint)samples.Length));
    }

    public void Commit(ulong id)
    {
        Guard();
        Check(N.sketch_stroke_commit(handle, id));
    }

    public PathCommand[] Paths()
    {
        Guard();
        Check(N.sketch_render_copy(handle, null, 0, out var count));
        var paths = new PathCommand[checked((int)count)];
        Check(N.sketch_render_copy(handle, paths, count, out _));
        return paths;
    }

    public void Select(double x, double y, double tolerance)
    {
        Guard();
        Check(N.sketch_select_at(handle, x, y, tolerance));
    }

    public void Cycle(int direction)
    {
        Guard();
        Check(N.sketch_select_cycle(handle, direction));
    }

    public void Resize(double scale, double x, double y)
    {
        Guard();
        Check(N.sketch_resize_update(handle, scale, x, y));
    }

    public void Save(string path)
    {
        Guard();
        var b = Encoding.UTF8.GetBytes(path);
        Check(N.sketch_document_save(handle, b, (nuint)b.Length));
    }

    public void Load(string path)
    {
        Guard();
        var b = Encoding.UTF8.GetBytes(path);
        Check(N.sketch_document_load(handle, b, (nuint)b.Length));
    }

    public void Command(string name)
    {
        Guard();
        Check(name switch
        {
            "cancel" => N.sketch_cancel(handle),
            "clear" => N.sketch_select_clear(handle),
            "delete" => N.sketch_delete_selected(handle),
            "resizeBegin" => N.sketch_resize_begin(handle),
            "resizeCommit" => N.sketch_resize_commit(handle),
            "undo" => N.sketch_undo(handle),
            "redo" => N.sketch_redo(handle),
            "new" => N.sketch_document_new(handle),
            _ => throw new ArgumentException(name)
        });
    }

    public void Dispose()
    {
        if (handle == IntPtr.Zero)
            return;
        if (owner != Environment.CurrentManagedThreadId)
            throw new InvalidOperationException("Wrong disposal thread");
        N.sketch_editor_free(handle);
        handle = IntPtr.Zero;
    }
    static class N
    {
        const string Lib = "sketch_ffi";
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)] internal static extern uint sketch_abi_version();
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)] internal static extern uint sketch_editor_create(out IntPtr editor);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)] internal static extern void sketch_editor_free(IntPtr editor);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)] internal static extern uint sketch_editor_state(IntPtr e, out State state);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)] internal static extern uint sketch_stroke_begin(IntPtr e, ulong id, uint device, Brush brush);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)] internal static extern uint sketch_stroke_append(IntPtr e, ulong id, [In] Sample[] samples, nuint count);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)] internal static extern uint sketch_stroke_commit(IntPtr e, ulong id);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)] internal static extern uint sketch_render_copy(IntPtr e, [Out] PathCommand[]? commands, nuint capacity, out nuint count);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)] internal static extern uint sketch_select_at(IntPtr e, double x, double y, double tolerance);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)] internal static extern uint sketch_select_cycle(IntPtr e, int direction);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)] internal static extern uint sketch_resize_update(IntPtr e, double scale, double x, double y);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)] internal static extern uint sketch_document_save(IntPtr e, byte[] path, nuint length);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)] internal static extern uint sketch_document_load(IntPtr e, byte[] path, nuint length);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)] internal static extern uint sketch_cancel(IntPtr e);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)] internal static extern uint sketch_select_clear(IntPtr e);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)] internal static extern uint sketch_delete_selected(IntPtr e);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)] internal static extern uint sketch_resize_begin(IntPtr e);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)] internal static extern uint sketch_resize_commit(IntPtr e);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)] internal static extern uint sketch_undo(IntPtr e);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)] internal static extern uint sketch_redo(IntPtr e);
        [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)] internal static extern uint sketch_document_new(IntPtr e);
    }
}
