using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;

namespace NativeSketch;

internal sealed class EditorView
{
    internal readonly Dictionary<string, MenuFlyoutItem> menuItems = new();
    internal readonly Grid root = new();
    internal readonly Canvas canvas = new()
    {
        Background = new SolidColorBrush(Microsoft.UI.Colors.White),
        IsTabStop = true
    };
    internal readonly TextBlock status = new()
    {
        Margin = new Thickness(12, 4, 12, 4)
    };
    internal readonly NumberBox width = new()
    {
        Header = "Width",
        Value = 4,
        Minimum = .1,
        Maximum = 256,
        Width = 105,
        SpinButtonPlacementMode = NumberBoxSpinButtonPlacementMode.Compact
    };
    internal readonly NumberBox scale = new()
    {
        Header = "Scale %",
        Value = 100,
        Minimum = 1,
        Maximum = 10000,
        Width = 110
    };
    internal readonly ColorPicker color = new()
    {
        Color = Microsoft.UI.Colors.Black,
        IsAlphaEnabled = true
    };
    internal readonly AppBarButton undo = new()
    {
        Label = "Undo",
        Icon = new SymbolIcon(Symbol.Undo)
    }
, redo = new()
{
    Label = "Redo",
    Icon = new SymbolIcon(Symbol.Redo)
}
, apply = new()
{
    Label = "Apply scale"
};
    internal readonly RadioButton pen = new()
    {
        Content = "Pen",
        IsChecked = true
    }
, select = new()
{
    Content = "Select"
};
}
