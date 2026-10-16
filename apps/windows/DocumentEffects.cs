using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Windows.Storage.Pickers;

namespace NativeSketch;

internal sealed class DocumentEffects
{
    readonly Window window;
    readonly Engine engine;
    readonly EditorView view;
    readonly Action Cancel;
    readonly Action Draw;
    internal bool busy, allowClose;
    internal string? file;
    internal DocumentEffects(Window window, Engine engine, EditorView view, Action cancel, Action draw)
    {
        this.window = window;
        this.engine = engine;
        this.view = view;
        Cancel = cancel;
        Draw = draw;
    }

    internal async Task Error(string message)
    {
        await new ContentDialog()
        {
            Title = "Native Sketch",
            Content = message,
            CloseButtonText = "OK",
            XamlRoot = view.root.XamlRoot
        }.ShowAsync();
    }

    async Task<bool> Save(bool choose)
    {
        string? path = file;
        if (path == null || choose)
        {
            var picker = new FileSavePicker()
            {
                SuggestedFileName = "Sketch"
            };
            picker.FileTypeChoices.Add("Native Sketch", new List<string> { ".nsketch" });
            WinRT.Interop.InitializeWithWindow.Initialize(picker, WinRT.Interop.WindowNative.GetWindowHandle(window));
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
        var result = await new ContentDialog()
        {
            Title = "Save changes?",
            Content = "This drawing has unsaved changes.",
            PrimaryButtonText = "Save",
            SecondaryButtonText = "Discard",
            CloseButtonText = "Cancel",
            DefaultButton = ContentDialogButton.Primary,
            XamlRoot = view.root.XamlRoot
        }.ShowAsync();
        return result == ContentDialogResult.Secondary || (result == ContentDialogResult.Primary && await Save(false));
    }

    internal async Task Document(string operation)
    {
        if (busy)
            return;
        Cancel();
        busy = true;
        view.canvas.IsHitTestVisible = false;
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
                    WinRT.Interop.InitializeWithWindow.Initialize(picker, WinRT.Interop.WindowNative.GetWindowHandle(window));
                    var target = await picker.PickSingleFileAsync();
                    if (target != null)
                    {
                        engine.Load(target.Path);
                        file = target.Path;
                    }
                }
            }
        }
        catch (Exception ex)
        {
            await Error(ex.Message);
        }
        finally
        {
            busy = false;
            view.canvas.IsHitTestVisible = true;
            Draw();
        }
    }

    internal async Task CloseSafely()
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
                window.Close();
            }
        }
        catch (Exception ex)
        {
            await Error(ex.Message);
        }
        finally
        {
            busy = false;
        }
    }
}
