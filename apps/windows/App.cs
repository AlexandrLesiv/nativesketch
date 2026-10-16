using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace NativeSketch;

public sealed class App : Application
{
    [STAThread]
    public static void Main()
    {
        WinRT.ComWrappersSupport.InitializeComWrappers();
        Application.Start(_ =>
        {
            SynchronizationContext.SetSynchronizationContext(new Microsoft.UI.Dispatching.DispatcherQueueSynchronizationContext(Microsoft.UI.Dispatching.DispatcherQueue.GetForCurrentThread()));
            new App();
        });
    }

    public App()
    {
        Resources.MergedDictionaries.Add(new XamlControlsResources());
    }
    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        new SketchWindow().Activate();
    }
}
