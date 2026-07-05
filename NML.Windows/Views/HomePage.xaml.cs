using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using System.Collections.ObjectModel;
using System.Threading.Tasks;

namespace NML.Windows.Views;

public sealed partial class HomePage : Page
{
    public ObservableCollection<VersionItem> VersionList { get; } = new();

    public HomePage()
    {
        this.InitializeComponent();
        LoadVersionsAsync();
    }

    private async void LoadVersionsAsync()
    {
        var core = App.Current.Services.GetService(typeof(Core.NMLCore)) as Core.NMLCore;
        if (core == null) return;
        
        try
        {
            var versions = await core.GetInstalledVersionsAsync();
            
            foreach (var v in versions)
            {
                VersionList.Add(new VersionItem { Id = v, Type = "正式版", ReleaseTime = "2024-01-01" });
            }
            
            VersionsGridView.ItemsSource = VersionList;
        }
        catch (Exception ex)
        {
            // TODO: Show error dialog
            System.Diagnostics.Debug.WriteLine($"Failed to load versions: {ex.Message}");
        }
    }

    private void LaunchButton_Click(object sender, RoutedEventArgs e)
    {
        // Launch selected version
    }

    private void LaunchVersionButton_Click(object sender, RoutedEventArgs e)
    {
        // Launch specific version
    }

    private void VersionsGridView_SelectionChanged(object sender, SelectionChangedEventArgs e)
    {
        // Handle selection
    }

    private void InstallRelease_Click(object sender, RoutedEventArgs e) { }
    private void InstallSnapshot_Click(object sender, RoutedEventArgs e) { }
    private void InstallForge_Click(object sender, RoutedEventArgs e) { }
    private void InstallFabric_Click(object sender, RoutedEventArgs e) { }
}

public class VersionItem
{
    public string Id { get; set; } = "";
    public string Type { get; set; } = "";
    public string ReleaseTime { get; set; } = "";
}
