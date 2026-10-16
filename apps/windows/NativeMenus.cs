using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Windows.System;

namespace NativeSketch;

internal sealed class NativeMenus
{
    readonly EditorView view;
    readonly Func<bool> isBusy;
    bool busy => isBusy();
    internal NativeMenus(EditorView view, Func<bool> isBusy)
    {
        this.view = view;
        this.isBusy = isBusy;
    }

    internal void Add(MenuBarItem parent, string text, Action action)
    {
        var item = new MenuFlyoutItem()
        {
            Text = text
        };
        item.Click += (_, _) =>
        {
            if (!busy)
                action();
        };
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
            var accelerator = new KeyboardAccelerator()
            {
                Key = key,
                Modifiers = VirtualKeyModifiers.Control | (text == "Save As…" ? VirtualKeyModifiers.Shift : VirtualKeyModifiers.None)
            };
            accelerator.Invoked += (_, e) =>
            {
                if ((text == "Undo" || text == "Redo") && FocusManager.GetFocusedElement(view.root.XamlRoot) is TextBox)
                    return;
                if (!busy && item.IsEnabled)
                    action();
                e.Handled = true;
            };
            item.KeyboardAccelerators.Add(accelerator);
        }
        parent.Items.Add(item);
        view.menuItems.Add(text, item);
    }
}
