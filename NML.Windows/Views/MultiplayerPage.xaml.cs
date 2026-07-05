using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace NML.Windows.Views;

public sealed partial class MultiplayerPage : Page
{
    public MultiplayerPage()
    {
        this.InitializeComponent();
    }

    private void HostButton_Click(object sender, RoutedEventArgs e)
    {
        var core = App.Current.Services.GetService(typeof(Core.NMLCore)) as Core.NMLCore;
        if (core == null) return;

        try
        {
            core.StartP2P();
            // TODO: Show hosting dialog
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"Failed to start P2P: {ex.Message}");
        }
    }

    private async void RefreshButton_Click(object sender, RoutedEventArgs e)
    {
        var core = App.Current.Services.GetService(typeof(Core.NMLCore)) as Core.NMLCore;
        if (core == null) return;

        try
        {
            var worlds = await core.DiscoverWorldsAsync();
            WorldsList.ItemsSource = worlds;
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"Failed to discover worlds: {ex.Message}");
        }
    }

    private void JoinButton_Click(object sender, RoutedEventArgs e)
    {
        // TODO: Join selected world
    }
}
